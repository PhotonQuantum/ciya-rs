use image::{DynamicImage, GenericImageView};
use opencv::core::{Mat, Vec3b};

use crate::errors::Result;

// Convert rust image to opencv mat.
// TODO: parallelize this conversion
pub fn img_to_mat(image: &DynamicImage) -> Result<Mat> {
    let (height, width) = (image.height() as usize, image.width() as usize);
    let bytes = image.clone().into_bgr8().into_raw();
    let mut mat_bytes: Vec<Vec<Vec3b>> = Vec::with_capacity(height);
    let mut ptr = bytes.as_ptr();
    for _ in 0..height {
        let mut row = Vec::with_capacity(width);
        for _ in 0..width {
            unsafe {
                let point = Vec3b::from([*ptr, *ptr.add(1), *ptr.add(2)]);
                row.push(point);
                ptr = ptr.add(3);
            }
        }
        mat_bytes.push(row);
    }
    Ok(Mat::from_slice_2d(&mat_bytes)?)
}
