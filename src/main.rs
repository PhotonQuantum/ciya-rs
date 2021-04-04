extern crate ciya_lib;

use anyhow::Result;
use image::io::Reader as ImageReader;

use ciya_lib::ciyafier::Ciyafier;
use show_image::create_window;
use std::env::args;

#[show_image::main]
fn main() -> Result<()> {
    println!("Initializing");
    let ciyafier = Ciyafier::new(true)?;
    println!("Reading file");
    let image = ImageReader::open(args().nth(1).unwrap()).unwrap().decode().unwrap();
    println!("Processing image");
    let image = ciyafier.ciya(image)?;
    println!("Displaying image");
    let window = create_window("result", Default::default())?;
    window.set_image("image", image)?;
    window.wait_until_destroyed()?;

    Ok(())
}
