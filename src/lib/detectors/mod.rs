use image::DynamicImage;
pub use weeb::WeebDetector;

use crate::errors::Result;
// pub use standard::StandardDetector;
use crate::types::ControlPoints;

mod weeb;

pub trait MouthDetectorTrait {
    fn detect(&self, image: &DynamicImage) -> Result<ControlPoints<f32>>;
}
