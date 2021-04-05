extern crate ciya_lib;

use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use clap::arg_enum;
use image::io::Reader as ImageReader;
use structopt::StructOpt;

use ciya_lib::ciyafier::{Ciyafier, Emotion};
use ciya_lib::detectors::WeebDetector;

mod resources;

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum Mode{
        Weeb,
        Standard
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum CliEmotion{
        Auto,
        Smile,
        Cry
    }
}

impl From<CliEmotion> for Emotion {
    fn from(v: CliEmotion) -> Self {
        match v {
            CliEmotion::Auto => Emotion::Auto,
            CliEmotion::Smile => Emotion::Smile,
            CliEmotion::Cry => Emotion::Cry
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "ciya-rs",
    about = "Ciyaify your image.",
    author = "LightQuantum <self@lightquantum.me>",
    version = "0.1.0"
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long, possible_values = & Mode::variants(), case_insensitive = true, default_value = "weeb")]
    mode: Mode,
    #[structopt(short, long, possible_values = & CliEmotion::variants(), case_insensitive = true, default_value = "auto")]
    emotion: CliEmotion,
    #[structopt(short, long, default_value = "8")]
    antialias_scale: u32
}

fn main() -> Result<()> {
    let opt: Opt = Opt::from_args();
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
