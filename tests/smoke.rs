use std::fs::File;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use image::ImageFormat;
use image::io::Reader as ImageReader;
use reqwest::blocking::Client;
use tempfile::tempdir;

use ciya_lib::ciyafier::{Ciyafier, Emotion};
use ciya_lib::detectors::WeebDetector;

const TEST_IMAGE: &[u8] = include_bytes!("test.png");

const FACE_MODEL_URL: &str = "https://raw.githubusercontent.com/nagadomi/lbpcascade_animeface/master/lbpcascade_animeface.xml";
const LANDMARK_MODEL_URL: &str =
    "https://github.com/PhotonQuantum/ciya-rs/releases/download/v0.0.1/anime_face_landmark.onnx";

#[test]
fn smoke_test() {
    let (face_model, landmark_model) = ensure_models();
    let detector = Box::new(
        WeebDetector::new(
            face_model.to_str().unwrap(),
            landmark_model.to_str().unwrap(),
        )
        .unwrap(),
    );
    let ciyafier = Ciyafier::new(detector);
    let image = ImageReader::with_format(Cursor::new(TEST_IMAGE), ImageFormat::Png).decode().unwrap();
    let _image = ciyafier
        .ciya(image, Emotion::Auto, 8)
        .unwrap();
}

fn ensure_models() -> (PathBuf, PathBuf) {
    let data_path = tempdir().unwrap().into_path();
    let download_path = (
        data_path.join("ciya-rs").join("lbpcascade_animeface.xml"),
        data_path.join("ciya-rs").join("anime_face_landmark.onnx"),
    );

    let http = Client::new();
    if !(download_path.0.is_file()) {
        let face_model = http.get(FACE_MODEL_URL).send().unwrap();
        ensure_dir(download_path.0.parent().unwrap());
        let mut file = File::create(&download_path.0).unwrap();
        file.write_all(&*face_model.bytes().unwrap()).unwrap();
    }
    if !(download_path.1.is_file()) {
        let landmark_model = http.get(LANDMARK_MODEL_URL).send().unwrap();
        ensure_dir(download_path.1.parent().unwrap());
        let mut file = File::create(&download_path.1).unwrap();
        file.write_all(&*landmark_model.bytes().unwrap()).unwrap();
    }
    download_path
}

fn ensure_dir(path: &Path) {
    if path.is_file() {
        panic!("Is a file")
    } else if !path.exists() {
        std::fs::create_dir_all(path).unwrap()
    }
}
