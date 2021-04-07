use std::io::Cursor;

use image::imageops::{self, FilterType};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use imageproc::geometric_transformations::{Interpolation, Projection};
use num::traits::Pow;
use num::{Num, NumCast};

use crate::errors::{Error, Result};
use crate::types::*;

const CIYA_RAW: &[u8] = include_bytes!("../../resources/ciya.png");

#[derive(Debug, Copy, Clone)]
pub enum Emotion {
    Auto,
    Smile,
    Cry,
}

pub struct Projector {
    ciya_image: RgbaImage,
    flipped_ciya_image: RgbaImage,
}

#[derive(Copy, Clone, Debug)]
enum ProjectionStrategy {
    Naive,
    RespectEdge { smile: bool },
}

impl Projector {
    pub fn new() -> Self {
        let mut image_raw = ImageReader::new(Cursor::new(CIYA_RAW));
        image_raw.set_format(ImageFormat::Png);
        let image = image_raw.decode().unwrap().into_rgba8();
        let flipped_image = imageops::flip_vertical(&image);
        Self {
            ciya_image: image,
            flipped_ciya_image: flipped_image,
        }
    }

    pub fn project(
        &self,
        mut image: DynamicImage,
        control_points: ControlPoints<f32>,
        emotion: Emotion,
        antialias_scale: u32,
    ) -> Result<DynamicImage> {
        let smile = is_smile(&control_points)
            .ok_or_else(|| Error::MathError(String::from("invalid control points")))?;

        // pick appropriate version of ciya according to emotion.
        let ciya = match emotion {
            Emotion::Auto => {
                if smile {
                    &self.ciya_image
                } else {
                    &self.flipped_ciya_image
                }
            }
            Emotion::Smile => &self.ciya_image,
            Emotion::Cry => &self.flipped_ciya_image,
        };

        let upscale_projection = Projection::scale(antialias_scale as f32, antialias_scale as f32);
        // calculate projection over ciya and overlay position in the target image
        let (offset, canvas_size, projection) = if !control_points
            .is_convex()
            .ok_or_else(|| Error::MathError(String::from("invalid control points")))?
        {
            // if ctrl_pts forms a concave quadrilateral, use the naive projection
            self.calc_ctrl_pts(control_points, ProjectionStrategy::Naive)
                .and_then(|(from, to)| proj_from_ctrl_pts(from, to))
                .ok_or_else(|| {
                    Error::MathError(String::from("unable to compute projection matrix"))
                })
        } else {
            // try to respect to detected mouth edges
            self.calc_ctrl_pts(control_points, ProjectionStrategy::RespectEdge { smile })
                .and_then(|(from, to)| proj_from_ctrl_pts(from, to))
                .or_else(|| {
                    // we can't form a valid projection matrix, fallback to naive projection
                    self.calc_ctrl_pts(control_points, ProjectionStrategy::Naive)
                        .and_then(|(from, to)| proj_from_ctrl_pts(from, to))
                })
                .ok_or_else(|| {
                    Error::MathError(String::from("unable to compute projection matrix"))
                })
        }
        .map(|(bound_lt, bound_rb, projection)| {
            (
                bound_lt,
                bound_rb - bound_lt,
                upscale_projection * projection, // upscale for antialiasing purpose
            )
        })?;

        let scaled_size = Point::<u32>::from(&canvas_size);
        // preallocate ciya canvas
        let mut warped_ciya = RgbaImage::new(
            scaled_size.x * antialias_scale,
            scaled_size.y * antialias_scale,
        );

        imageproc::geometric_transformations::warp_into(
            ciya,
            &projection,
            Interpolation::Bicubic,
            Rgba([0, 0, 0, 0]),
            &mut warped_ciya,
        );

        // downscale to target size
        let warped_ciya = image::imageops::resize(
            &warped_ciya,
            scaled_size.x,
            scaled_size.y,
            FilterType::Lanczos3,
        );
        image::imageops::overlay(&mut image, &warped_ciya, offset.x as u32, offset.y as u32);
        Ok(image)
    }

    fn calc_ctrl_pts(
        &self,
        control_points: ControlPoints<f32>,
        strategy: ProjectionStrategy,
    ) -> Option<(ControlPoints<f32>, ControlPoints<f32>)> {
        let (ciya_ctrl_pts, target_ctrl_pts) = match strategy {
            ProjectionStrategy::Naive => (
                ControlPoints::from(&Rectangle::new(
                    0.,
                    0.,
                    self.ciya_image.width() as f32,
                    self.ciya_image.height() as f32,
                )),
                control_points.centralize_y().enlarge(0.3, true),
            ),
            ProjectionStrategy::RespectEdge { smile } => {
                let target_ctrl_pts = control_points.enlarge(0.3, false);
                let ciya_ctrl_pts = {
                    let y0 = target_ctrl_pts.cross().y;
                    let factor =
                        (y0 - target_ctrl_pts.p2.y) / (target_ctrl_pts.p4.y - target_ctrl_pts.p2.y);
                    let y = 200. * factor;

                    // \frac{(x-180)^2}{180^2} + \frac{y^2}{200^2} = 1
                    let d = if smile {
                        (1. - y.pow(2.) / 40000.).sqrt()
                    } else {
                        let y = 200. - y;
                        (1. - y.pow(2.) / 40000.).sqrt()
                    };

                    let x1 = 180. * (1. - d);
                    let x2 = 180. * (1. + d);

                    ControlPoints::new(
                        Point::new(x1, y),
                        Point::new(self.ciya_image.width() as f32 / 2., 0.),
                        Point::new(x2, y),
                        Point::new(
                            self.ciya_image.width() as f32 / 2.,
                            self.ciya_image.height() as f32,
                        ),
                    )
                };

                (ciya_ctrl_pts, target_ctrl_pts)
            }
        };

        if ciya_ctrl_pts.is_irregular() || target_ctrl_pts.is_irregular() {
            None
        } else {
            Some((ciya_ctrl_pts, target_ctrl_pts))
        }
    }
}

fn is_smile<T: Num + NumCast + PartialOrd + Copy>(
    control_points: &ControlPoints<T>,
) -> Option<bool> {
    let Point { x: _, y: y0 } = control_points.cross();
    Some(user_abs_minus(control_points.p2.y, y0)? <= user_abs_minus(control_points.p4.y, y0)?)
}

fn proj_from_ctrl_pts(
    ciya_ctrl_pts: ControlPoints<f32>,
    target_ctrl_pts: ControlPoints<f32>,
) -> Option<(Point<f32>, Point<f32>, Projection)> {
    target_ctrl_pts
        .shift_origin()
        .and_then(|(bound_lt, bound_rb, ref base_landmarks)| {
            Projection::from_control_points((&ciya_ctrl_pts).into(), base_landmarks.into())
                .map(|projection| (bound_lt, bound_rb, projection))
        })
}
