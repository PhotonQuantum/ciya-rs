extern crate ciya_lib;

use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use ciya_lib::{
    ciyafier::{Ciyafier, Emotion},
    detectors::WeebDetector,
};
use clap::{Parser, ValueEnum};
use image::io::Reader as ImageReader;

mod resources;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Mode {
    Weeb,
    Standard,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum CliEmotion {
    Auto,
    Smile,
    Cry,
}

impl From<CliEmotion> for Emotion {
    fn from(v: CliEmotion) -> Self {
        match v {
            CliEmotion::Auto => Self::Auto,
            CliEmotion::Smile => Self::Smile,
            CliEmotion::Cry => Self::Cry,
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(name = "ciya-cli")]
#[command(author, version, about)]
struct Opt {
    input: PathBuf,
    output: PathBuf,
    #[arg(short, long, value_enum, default_value_t = Mode::Weeb)]
    mode: Mode,
    #[arg(short, long, value_enum, default_value_t = CliEmotion::Auto)]
    emotion: CliEmotion,
    #[arg(short, long, default_value_t = 8)]
    antialias_scale: u32,
}

fn main() -> Result<()> {
    let opt: Opt = Opt::parse();
    let detector = match opt.mode {
        Mode::Weeb => {
            let (face_model, landmark_model) = resources::ensure_models()?;
            Box::new(WeebDetector::new(
                face_model
                    .to_str()
                    .ok_or_else(|| anyhow!("some path thing error"))?,
                landmark_model
                    .to_str()
                    .ok_or_else(|| anyhow!("some path thing error"))?,
            )?)
        }
        Mode::Standard => {
            bail!("Standard mode not implemented")
        }
    };
    println!("Initializing");
    let ciyafier = Ciyafier::new(detector);
    println!("Reading file");
    let image = ImageReader::open(opt.input).unwrap().decode().unwrap();
    println!("Processing image");
    let image = ciyafier.ciya(image, opt.emotion.into(), opt.antialias_scale)?;
    println!("Writing image");
    image.save(opt.output)?;

    Ok(())
}
