extern crate ciya_lib;

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{anyhow, Result};
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageFormat};
use structopt::StructOpt;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, InputFile, Message, PhotoSize};

use ciya_lib::ciyafier::Ciyafier;
use ciya_lib::detectors::WeebDetector;
use ciya_lib::errors::Error;

use crate::commands::{Command, Mode, Opt};
use crate::resources::ensure_models;

mod commands;
mod resources;

fn best_photos(photos: &[PhotoSize]) -> Vec<&PhotoSize> {
    let mut unique_photos: HashMap<&str, &PhotoSize> = HashMap::new();
    for photo in photos {
        if let Some(photo_in_map) = unique_photos.get(photo.file_unique_id.as_str()) {
            if photo.height > photo_in_map.height {
                unique_photos.insert(photo.file_unique_id.as_str(), photo);
            }
        } else {
            unique_photos.insert(photo.file_unique_id.as_str(), photo);
        }
    }
    unique_photos.into_values().collect()
}

fn image_from_message(message: &Message) -> Option<&str> {
    message
        .document()
        .and_then(|doc| {
            doc.mime_type.as_ref().and_then(|_mime| {
                if _mime.type_() == mime::IMAGE {
                    Some(doc.file_id.as_str())
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            message
                .photo()
                .and_then(|photos| Some(best_photos(photos).into_iter().next()?.file_id.as_str()))
        })
        .or_else(|| {
            message.sticker().and_then(|sticker| {
                if !sticker.is_animated {
                    Some(sticker.file_id.as_str())
                } else {
                    None
                }
            })
        })
}

fn decode_image(bytes: &[u8]) -> Result<DynamicImage> {
    let guessed_image = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    if let Some(ImageFormat::WebP) = guessed_image.format() {
        let decoder = webp::Decoder::new(bytes);
        Ok(decoder
            .decode()
            .ok_or_else(|| anyhow!("some decode thing failed"))?
            .to_image())
    } else {
        Ok(guessed_image.decode()?)
    }
    .and_then(|image| {
        if image.width() > 4096 || image.height() > 4096 {
            Err(anyhow!("Image too large"))
        } else {
            Ok(image)
        }
    })
}

async fn answer(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Command) -> Result<()> {
    match command {
        Command::Help | Command::Start => {
            let mut buf: Vec<u8> = Vec::new();
            Opt::clap().write_long_help(&mut buf)?;
            cx.answer(String::from_utf8(buf)?).await?
        }
        Command::Ciyaify(opt) => match opt {
            Err(err) => cx.answer(format!("{:#}", err)).await?,
            Ok(opt) => {
                let message = cx.update.reply_to_message();
                match message {
                    None => {
                        cx.answer("Please reply to the image you want to ciyaify.")
                            .await?
                    }
                    Some(message) => {
                        let file_id = image_from_message(message);
                        match file_id {
                            None => {
                                cx.answer("Please reply to the image you want to ciyaify.")
                                    .await?
                            }
                            Some(file_id) => {
                                if opt.antialias_scale > 8 {
                                    cx.answer("antialias_scale must <= 8.").await?
                                } else {
                                    #[allow(unused_must_use)]
                                    let _ = {
                                        cx.requester
                                            .send_chat_action(cx.chat_id(), ChatAction::Typing)
                                            .await;
                                    };

                                    let mut buffer = Vec::new();
                                    cx.requester
                                        .download_file(
                                            &cx.requester.get_file(file_id).await?.file_path,
                                            &mut buffer,
                                        )
                                        .await?;
                                    let models = ensure_models();
                                    match models {
                                        None => cx.answer("Unable to load model.").await?,
                                        Some((face_model, landmark_model)) => {
                                            match decode_image(&*buffer) {
                                                Err(_) => {
                                                    cx.answer("Invalid image format.").await?
                                                }
                                                Ok(image) => {
                                                    let output = {
                                                        let detector = match opt.mode {
                                                            Mode::Weeb => Box::new(WeebDetector::new(
                                                                face_model.to_str().ok_or_else(|| {
                                                                    anyhow!("some path thing failed")
                                                                })?,
                                                                landmark_model.to_str().ok_or_else(|| {
                                                                    anyhow!("some path thing failed")
                                                                })?,
                                                            )?),
                                                            Mode::Standard => {
                                                                cx.answer("Standard detector not implemented.").await?;
                                                                return Ok(());
                                                            }
                                                        };
                                                        let ciyaify = Ciyafier::new(detector);
                                                        ciyaify.ciya(
                                                            image,
                                                            opt.emotion.into(),
                                                            opt.antialias_scale,
                                                        )
                                                    };
                                                    match output {
                                                        Err(Error::NoneError) => {
                                                            cx.answer("No face or mouth detected.")
                                                                .await?
                                                        }
                                                        Err(err) => {
                                                            cx.answer(format!("{}", err)).await?
                                                        }
                                                        Ok(output) => {
                                                            let encoder =
                                                                webp::Encoder::from_image(&output)
                                                                    .unwrap();
                                                            let bytes: Vec<u8> =
                                                                (*encoder.encode(80.)).to_vec();
                                                            cx.answer_document(InputFile::Memory {
                                                                file_name: String::from(
                                                                    "ciya.webp",
                                                                ),
                                                                data: Cow::from(bytes),
                                                            })
                                                            .await?
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting ciya_bot...");

    let bot = Bot::from_env().auto_send();

    let bot_name: String = "ciyaify_bot".to_string();
    teloxide::commands_repl(bot, bot_name, answer).await;
}
