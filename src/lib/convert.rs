use image::{ImageBuffer, Rgb};
use opencv::core::{DataType, Mat, MatTraitConst, Mat_AUTO_STEP, CV_MAKETYPE};

use crate::errors::Result;

// Convert rgb image to opencv bgr mat zero-copy.
pub fn img_to_mat(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Mat> {
    let (width, height) = image.dimensions();
    let cv_type = CV_MAKETYPE(u8::depth(), 3);
    let mat = {
        // SAFETY: this creates a reference to the underlying data of the image buffer.
        // However, it doesn't survive outside of this block.
        let rgb = unsafe {
            Mat::new_rows_cols_with_data(
                height as i32,
                width as i32,
                cv_type,
                image.as_ptr() as *mut _,
                Mat_AUTO_STEP,
            )
        }?
        .try_clone()?;
        let mut bgr = Mat::default();
        opencv::imgproc::cvt_color(&rgb, &mut bgr, opencv::imgproc::COLOR_RGB2BGR, 3)?;
        bgr
    };
    Ok(mat)
}
