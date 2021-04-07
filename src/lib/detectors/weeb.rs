use std::cell::RefCell;
use std::cmp::{max, min, Ordering};
use std::convert::TryInto;

use image::imageops::FilterType;
use image::DynamicImage;
use itertools::Itertools;
use lazy_static::lazy_static;
use ndarray::parallel::prelude::*;
use ndarray::{Array, ArrayBase, Axis, Ix3, Ix4, OwnedRepr, RemoveAxis, ViewRepr};
use ndarray_image::{NdColor, NdImage};
use num::{Num, NumCast};
use onnxruntime::environment::Environment;
use onnxruntime::session::Session;
use onnxruntime::tensor::OrtOwnedTensor;
use opencv::core::{Rect, Size};
use opencv::objdetect::{CascadeClassifier, CascadeClassifierTrait};
use opencv::prelude::*;
use opencv::types::VectorOfRect;

use crate::convert::img_to_mat;
use crate::detectors::MouthDetectorTrait;
use crate::errors::{Error, Result};
use crate::types::*;

lazy_static! {
    static ref ENV: Environment = Environment::builder()
        .with_name("anime_landmark_detector")
        .build()
        .unwrap();
}

pub struct WeebDetector<'a> {
    face_detector: RefCell<CascadeClassifier>,
    landmark_detector: RefCell<onnxruntime::session::Session<'a>>,
}

impl<'a> WeebDetector<'a> {
    pub fn new(face_model: &str, landmark_model: &str) -> Result<Self> {
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
                    .crop_imm(face_rect.x, face_rect.y, face_rect.w, face_rect.h)
                    .resize_exact(128, 128, FilterType::Lanczos3)
                    .into_rgb8();
                let face_array = Into::<NdColor>::into(NdImage(&face))
                    .permuted_axes([2, 0, 1])
                    .mapv(|i| i as f32);

                // normalize matrix
                let nn_input = face_to_nn_input(face_array);

                // predict landmarks using pretrained model (onnxruntime)
                let mut landmark_detector = self.landmark_detector.borrow_mut();
                let nn_outputs: Vec<OrtOwnedTensor<f32, _>> =
                    landmark_detector.run(vec![nn_input])?;

                // extract the latest stage
                let nn_output = nn_outputs.into_iter().last().unwrap();
                let heatmap = nn_output.index_axis(Axis(0), 0);

                // find the most probable coords for mouth landmarks from heatmap
                let landmarks: Vec<_> = heatmap
                    .axis_iter(Axis(0))
                    .dropping(20)
                    .take(4)
                    .map(argmax)
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
            .flatten()
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

fn argmax<T: RemoveAxis>(array: ArrayBase<ViewRepr<&f32>, T>) -> (usize, usize) {
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
