use std::{io, result};

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cv error: {0}")]
    CVError(#[from] opencv::Error),
    #[error("onnxruntime error: {0}")]
    OrtError(#[from] onnxruntime::OrtError),
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("math error: {0}")]
    MathError(String),
    #[error("internal error for None")]
    NoneError,
}
