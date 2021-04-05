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
        let smiling = is_smile(&control_points);
        let ciya = match emotion {
            Emotion::Auto => {
                if smiling {
                    &self.ciya_image
                } else {
                    &self.flipped_ciya_image
                }
            }
            Emotion::Smile => &self.ciya_image,
            Emotion::Cry => &self.flipped_ciya_image,
        };

        let (bound_lt, bound_rb, projection) = if !control_points.is_convex() {
            let (ciya_ctrl_pts, target_ctrl_pts) = self.calc_ctrl_pts(control_points, true, true);
            let (bound_lt, bound_rb, base_landmarks) = target_ctrl_pts.shift_origin();
            let projection = Projection::scale(antialias_scale as f32, antialias_scale as f32)
                * Projection::from_control_points(
                    (&ciya_ctrl_pts).into(),
                    (&base_landmarks).into(),
                )
                .ok_or_else(|| {
                    Error::MathError(String::from("unable to compute projection matrix"))
                })?;
            (bound_lt, bound_rb, projection)
        } else {
            let (ciya_ctrl_pts, target_ctrl_pts) =
                self.calc_ctrl_pts(control_points, false, smiling);
            let (bound_lt, bound_rb, base_landmarks) = target_ctrl_pts.shift_origin();
            let upscale_projection =
                Projection::scale(antialias_scale as f32, antialias_scale as f32);
            Projection::from_control_points((&ciya_ctrl_pts).into(), (&base_landmarks).into())
                .map(|projection| (bound_lt, bound_rb, upscale_projection * projection))
                .or_else(|| {
                    // fallback to naive projection
                    let (ciya_ctrl_pts, target_ctrl_pts) =
                        self.calc_ctrl_pts(control_points, true, true);
                    let (bound_lt, bound_rb, base_landmarks) = target_ctrl_pts.shift_origin();
                    let projection = Projection::from_control_points(
                        (&ciya_ctrl_pts).into(),
                        (&base_landmarks).into(),
                    )?;
                    Some((bound_lt, bound_rb, projection))
                })
                .map(|(bound_lt, bound_rb, projection)| {
                    (bound_lt, bound_rb, upscale_projection * projection)
                })
                .ok_or_else(|| {
                    Error::MathError(String::from("unable to compute projection matrix"))
                })?
        };

        let rebased_rb = Point::<u32>::from(&(bound_rb - bound_lt));
        let mut warped_ciya = RgbaImage::new(
            rebased_rb.x * antialias_scale,
            rebased_rb.y * antialias_scale,
        );
        imageproc::geometric_transformations::warp_into(
            ciya,
            &projection,
            Interpolation::Bicubic,
            Rgba([0, 0, 0, 0]),
            &mut warped_ciya,
        );
        let warped_ciya = image::imageops::resize(
            &warped_ciya,
            rebased_rb.x,
            rebased_rb.y,
            FilterType::Lanczos3,
        );
        image::imageops::overlay(
            &mut image,
            &warped_ciya,
            bound_lt.x as u32,
            bound_lt.y as u32,
        );
        Ok(image)
    }

    fn calc_ctrl_pts(
        &self,
        control_points: ControlPoints<f32>,
        naive: bool,
        smiling: bool,
    ) -> (ControlPoints<f32>, ControlPoints<f32>) {
        if naive {
            (
                ControlPoints::from(&Rectangle::new(
                    0.,
                    0.,
                    self.ciya_image.width() as f32,
                    self.ciya_image.height() as f32,
                )),
                control_points.centralize_y().enlarge(0.3, true),
            )
        } else {
            let target_ctrl_pts = control_points.enlarge(0.3, false);
            let ciya_ctrl_pts = {
                let y0 = target_ctrl_pts.cross().y;
                let factor =
                    (y0 - target_ctrl_pts.p2.y) / (target_ctrl_pts.p4.y - target_ctrl_pts.p2.y);
                let y = 200. * factor;

                // \frac{(x-180)^2}{180^2} + \frac{y^2}{200^2} = 1
                let d = if smiling {
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
    }
}

fn is_smile<T: Num + NumCast + PartialOrd + Copy>(control_points: &ControlPoints<T>) -> bool {
    let Point { x: x0, y: _ } = control_points.cross();
    user_abs_minus(control_points.p2.x, x0) <= user_abs_minus(control_points.p4.x, x0)
}
