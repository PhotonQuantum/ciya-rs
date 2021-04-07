use image::DynamicImage;
use opencv::core::Mat;
use opencv::imgcodecs;
use tempfile::tempdir;

use crate::errors::Result;

// a naive implementation to convert rust image to opencv mat.
// use file as a buffer. may have a better implementation.
pub fn img_to_mat(img: &DynamicImage) -> Result<Mat> {
    let dir = tempdir()?;
    let filename = dir.path().join("temp.bmp");

    img.save(&filename)?;
    let mat = imgcodecs::imread(filename.to_str().unwrap(), imgcodecs::IMREAD_COLOR)?;

    dir.close()?;
    Ok(mat)
}
