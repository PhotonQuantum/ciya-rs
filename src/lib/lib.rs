#![allow(
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::default_trait_access
)]

#[macro_use]
mod types;
pub mod ciyafier;
mod convert;
pub mod detectors;
pub mod errors;
mod projector;
