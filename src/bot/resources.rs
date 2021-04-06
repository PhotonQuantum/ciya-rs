use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;

const FACE_MODEL_URL: &str = "https://raw.githubusercontent.com/nagadomi/lbpcascade_animeface/master/lbpcascade_animeface.xml";
const LANDMARK_MODEL_URL: &str =
    "https://github.com/PhotonQuantum/ciya-rs/releases/download/v0.0.1/anime_face_landmark.onnx";

static mut MODELS: Option<(PathBuf, PathBuf)> = None;
static INIT: Once = Once::new();

pub fn ensure_models() -> &'static Option<(PathBuf, PathBuf)> {
    unsafe {
        INIT.call_once(|| {
            MODELS = _ensure_models().ok();
        });
        &MODELS
    }
}

fn _ensure_models() -> Result<(PathBuf, PathBuf)> {
    let current_dir = std::env::current_dir()?;
    let data_dir = dirs::data_local_dir().ok_or_else(|| anyhow!("Missing data dir"))?;
    let local_path = (
        current_dir.join("lbpcascade_animeface.xml"),
        current_dir.join("anime_face_landmark.onnx"),
    );
    let download_path = (
        data_dir.join("ciya-rs").join("lbpcascade_animeface.xml"),
        data_dir.join("ciya-rs").join("anime_face_landmark.onnx"),
    );

    if local_path.0.is_file() && local_path.1.is_file() {
        Ok(local_path)
    } else if download_path.0.is_file() && download_path.1.is_file() {
        Ok(download_path)
    } else {
        let http = Client::new();
        if !(download_path.0.is_file()) {
            let face_model = http.get(FACE_MODEL_URL).send()?;
            ensure_dir(download_path.0.parent().unwrap())?;
            let mut file = File::create(&download_path.0)?;
            file.write_all(&*face_model.bytes()?)?;
        }
        if !(download_path.1.is_file()) {
            let landmark_model = http.get(LANDMARK_MODEL_URL).send()?;
            ensure_dir(download_path.1.parent().unwrap())?;
            let mut file = File::create(&download_path.1)?;
            file.write_all(&*landmark_model.bytes()?)?;
        }
        Ok(download_path)
    }
}

fn ensure_dir(path: &Path) -> Result<()> {
    if path.is_file() {
        Err(anyhow!("Is a file"))
    } else if path.exists() {
        Ok(())
    } else {
        Ok(std::fs::create_dir_all(path)?)
    }
}
