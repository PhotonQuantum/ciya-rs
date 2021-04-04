use crate::detectors::{MouthDetectorTrait, WeebDetector};
use crate::errors::Result;
use crate::projector::Projector;
use image::DynamicImage;

pub struct Ciyafier {
    detector: Box<dyn MouthDetectorTrait>,
    projector: Projector,
}

impl Ciyafier {
    pub fn new(weeb: bool) -> Result<Self> {
        assert_eq!(weeb, true, "non-weeb detector not implemented");
        Ok(Self {
            detector: Box::new(WeebDetector::new()?),
            projector: Projector::new(),
        })
    }
    pub fn ciya(&self, image: DynamicImage) -> Result<DynamicImage> {
        let control_points = self.detector.detect(&image)?;
        self.projector.project(image, control_points, 8)
    }
}
