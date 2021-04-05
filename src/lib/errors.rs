use std::{io, result};

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cv error")]
    CVError(#[from] opencv::Error),
    #[error("onnxruntime error")]
    OrtError(#[from] onnxruntime::OrtError),
    #[error("image error")]
    ImageError(#[from] image::ImageError),
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("math error")]
    MathError(String),
    #[error("internal error for None")]
    NoneError,
}
