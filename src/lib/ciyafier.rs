use image::DynamicImage;

use crate::detectors::MouthDetectorTrait;
use crate::errors::Result;
use crate::projector::Projector;
pub use crate::projector::Emotion;

pub struct Ciyafier {
    detector: Box<dyn MouthDetectorTrait>,
    projector: Projector,
}

impl Ciyafier {
    pub fn new(detector: Box<dyn MouthDetectorTrait>) -> Self {
        Self {
            detector,
            projector: Projector::new(),
        }
    }
    pub fn ciya(&self, image: DynamicImage, emotion: Emotion, antialias_scale: u32) -> Result<DynamicImage> {
        let control_points = self.detector.detect(&image)?;
        self.projector.project(image, control_points, emotion, antialias_scale)
    }
}