use crate::errors::Result;

// pub use standard::StandardDetector;
use crate::types::ControlPoints;
use image::DynamicImage;
pub use weeb::WeebDetector;

mod weeb;

pub trait MouthDetectorTrait {
    fn detect(&self, image: &DynamicImage) -> Result<ControlPoints<f32>>;
}
