use image::DynamicImage;

pub use crate::projector::Emotion;
use crate::{detectors::MouthDetectorTrait, errors::Result, projector::Projector};

pub struct Ciyafier {
    detector: Box<dyn MouthDetectorTrait>,
    projector: Projector,
}

impl Ciyafier {
    #[must_use]
    pub fn new(detector: Box<dyn MouthDetectorTrait>) -> Self {
        Self {
            detector,
            projector: Projector::new(),
        }
    }

    pub fn ciya(
        &self,
        image: DynamicImage,
        emotion: Emotion,
        antialias_scale: u32,
    ) -> Result<DynamicImage> {
        let control_points = self.detector.detect(&image)?;
        self.projector
            .project(image, control_points, emotion, antialias_scale)
    }
}
