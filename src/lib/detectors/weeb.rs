use std::{
    cell::RefCell,
    cmp::{max, min, Ordering},
    convert::TryInto,
};

use image::{imageops, imageops::FilterType, DynamicImage};
use itertools::Itertools;
use lazy_static::lazy_static;
use mcai_onnxruntime::{
    environment::Environment,
    session::Session,
    tensor::{FromArray, InputTensor, OrtOwnedTensor},
};
use ndarray::{
    parallel::prelude::*,
    Array,
    ArrayBase,
    Axis,
    Ix3,
    Ix4,
    OwnedRepr,
    RemoveAxis,
    ViewRepr,
};
use nshare::ToNdarray3;
use num::{Num, NumCast};
use opencv::{
    core::{Rect, Size},
    objdetect::{CascadeClassifier, CascadeClassifierTrait},
    prelude::*,
    types::VectorOfRect,
};
use tap::Pipe;

use crate::{
    convert::img_to_mat,
    detectors::MouthDetectorTrait,
    errors::{Error, Result},
    types::{ControlPoints, Point, Rectangle},
};

lazy_static! {
    static ref ENV: Environment = Environment::builder()
        .with_name("anime_landmark_detector")
        .build()
        .unwrap();
}

pub struct WeebDetector<'a> {
    face_detector: RefCell<CascadeClassifier>,
    landmark_detector: RefCell<Session<'a>>,
}

impl<'a> WeebDetector<'a> {
    pub fn new(face_model: &str, landmark_model: &str) -> Result<Self> {
        #[allow(clippy::unnecessary_to_owned)]
        let session: Session = ENV
            .new_session_builder()?
            .with_model_from_file(landmark_model.to_string())?;
        Ok(Self {
            face_detector: RefCell::new(CascadeClassifier::new(face_model).unwrap()),
            landmark_detector: RefCell::new(session),
        })
    }
}

impl MouthDetectorTrait for WeebDetector<'_> {
    fn detect(&self, image: &DynamicImage) -> Result<ControlPoints<f32>> {
        let buffer;
        #[allow(clippy::option_if_let_else)]
        let image = if let Some(image) = image.as_rgb8() {
            image
        } else {
            buffer = image.to_rgb8();
            &buffer
        };
        // convert rust image to matrix
        let image_mat = img_to_mat(image)?;

        // detect face position using pretrained cascade classifier
        let mut cv_faces = VectorOfRect::new();
        self.face_detector.borrow_mut().detect_multi_scale(
            &image_mat,
            &mut cv_faces,
            1.1,
            3,
            0,
            Size::new(0, 0),
            Size::new(0, 0),
        )?;

        let cv_faces = cv_faces.to_vec();
        (!cv_faces.is_empty())
            .then(|| -> Result<_> {
                // find largest face and slightly enlarge roi
                let cv_face = cv_faces.iter().max_by_key(|rect| rect.area()).unwrap();
                let face_rect = face_to_roi(image_mat.size()?.width, cv_face);

                // crop image and convert into matrix
                let face = image
                    .pipe(|img| {
                        imageops::crop_imm(img, face_rect.x, face_rect.y, face_rect.w, face_rect.h)
                    })
                    .pipe(|img| imageops::resize(&*img, 128, 128, FilterType::Lanczos3));
                let face_array = face.into_ndarray3().mapv(|i| i as f32);

                // normalize matrix
                let nn_input = face_to_nn_input(face_array);
                let input_tensor = InputTensor::from_array(nn_input);

                // predict landmarks using pretrained model (onnxruntime)
                let mut landmark_detector = self.landmark_detector.borrow_mut();
                let nn_outputs: Vec<OrtOwnedTensor<f32, _>> =
                    landmark_detector.run(vec![input_tensor])?;

                // extract the latest stage
                let nn_output = nn_outputs.into_iter().last().unwrap();
                let heatmap = nn_output.index_axis(Axis(0), 0);

                // find the most probable coords for mouth landmarks from heatmap
                let landmarks: Vec<_> = heatmap
                    .axis_iter(Axis(0))
                    .dropping(20)
                    .take(4)
                    .map(|x| argmax(&x))
                    .map(|(x, y)| Point::new(x as u32, y as u32))
                    .collect();

                // rebase the coords
                let base_landmarks: ControlPoints<f32> = landmarks
                    .iter()
                    .map(|point| rebase(*point, &Rectangle::new(0, 0, 128, 128), &face_rect))
                    .map(|point| Point::new(point.x as f32, point.y as f32))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                Ok(base_landmarks)
            })
            .ok_or(Error::NoneError)
            .and_then(|x| x) // .flatten() is gated by `result_flattening`
    }
}

fn rebase<T: Num + NumCast + PartialOrd + Copy>(
    coord: Point<T>,
    from: &Rectangle<T>,
    to: &Rectangle<T>,
) -> Point<T> {
    Point {
        x: to.x
            + (num::cast(
                (num::cast::<_, f32>(coord.x).unwrap()) * (num::cast::<_, f32>(to.w).unwrap())
                    / (num::cast::<_, f32>(from.w).unwrap()),
            )
            .unwrap()),
        y: to.y
            + (num::cast(
                (num::cast::<_, f32>(coord.y).unwrap()) * (num::cast::<_, f32>(to.h).unwrap())
                    / (num::cast::<_, f32>(from.h).unwrap()),
            )
            .unwrap()),
    }
}

fn argmax<T: RemoveAxis>(array: &ArrayBase<ViewRepr<&f32>, T>) -> (usize, usize) {
    array
        .axis_iter(Axis(0))
        .into_par_iter()
        .map(|axis_y_1d| {
            axis_y_1d
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .map(|(id, v)| (id, *v))
                .unwrap()
        })
        .enumerate()
        .max_by(|(_, (_, a)), (_, (_, b))| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(x, (y, _))| (y, x))
        .unwrap()
}

fn face_to_nn_input(
    mut face_array: ArrayBase<OwnedRepr<f32>, Ix3>,
) -> ArrayBase<OwnedRepr<f32>, Ix4> {
    let min_diff: Vec<_> = face_array
        .axis_iter(Axis(0))
        .into_par_iter()
        .map(|channel| {
            let (min, max) = channel
                .iter()
                .minmax_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
                .into_option()
                .unwrap();
            let diff = max - min;
            (*min, diff)
        })
        .collect();
    for (mut channel, (min, diff)) in face_array.axis_iter_mut(Axis(0)).zip(min_diff) {
        channel.iter_mut().for_each(|v| {
            *v = (*v - min) / diff * 2. - 1.;
        });
    }

    let face_array = face_array.insert_axis(Axis(0));
    let mut output = Array::default(face_array.raw_dim());
    output.assign(&face_array);
    output
}

fn face_to_roi<T: Num + NumCast + PartialOrd + Copy>(
    universe_width: i32,
    face: &Rect,
) -> Rectangle<T> {
    let Rect {
        x: x_,
        y: y_,
        width: w_,
        height: h_,
    } = face;
    let x = max(x_ - w_ / 8, 0);
    let rx = min(x_ + w_ * 9 / 8, universe_width);
    let y = max(y_ - h_ / 4, 0);
    let by = y_ + h_;
    let w = rx - x;
    let h = by - y;
    Rectangle::new(cast!(x), cast!(y), cast!(w), cast!(h))
}
