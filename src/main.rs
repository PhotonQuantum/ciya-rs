extern crate ciya_lib;

use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use clap::arg_enum;
use image::io::Reader as ImageReader;
use structopt::StructOpt;

use ciya_lib::ciyafier::Ciyafier;
use ciya_lib::detectors::WeebDetector;

mod resources;

arg_enum! {
    #[derive(Debug)]
    enum Mode{
        Weeb,
        Standard
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ciya-rs",
    about = "Ciyaify your image.",
    author = "LightQuantum",
    version = "0.1.0"
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(possible_values = & Mode::variants(), case_insensitive = true, default_value = "weeb")]
    mode: Mode,
}

fn main() -> Result<()> {
    let opt: Opt = Opt::from_args();
    let detector = match opt.mode {
        Mode::Weeb => {
            let (face_model, landmark_model) = resources::ensure_models()?;
            WeebDetector::new(
                face_model
                    .to_str()
                    .ok_or_else(|| anyhow!("some path thing error"))?,
                landmark_model
                    .to_str()
                    .ok_or_else(|| anyhow!("some path thing error"))?,
            )?
        }
        Mode::Standard => {
            bail!("Standard mode not implemented")
        }
    };
    println!("Initializing");
    let ciyafier = Ciyafier::new(Box::new(detector));
    println!("Reading file");
    let image = ImageReader::open(opt.input).unwrap().decode().unwrap();
    println!("Processing image");
    let image = ciyafier.ciya(image)?;
    println!("Writing image");
    image.save(opt.output)?;

    Ok(())
}
