use image::DynamicImage;

use crate::detectors::MouthDetectorTrait;
use crate::errors::Result;
use crate::projector::Projector;

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
    pub fn ciya(&self, image: DynamicImage) -> Result<DynamicImage> {
        let control_points = self.detector.detect(&image)?;
        self.projector.project(image, control_points, 8)
    }
}
