extern crate ciya_lib;

use std::{collections::HashMap, io::Cursor};

use anyhow::{anyhow, Result};
use ciya_lib::{ciyafier::Ciyafier, detectors::WeebDetector, errors::Error};
use clap::CommandFactory;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use log::{info, warn};
use teloxide::{
    net::Download,
    prelude::*,
    types::{ChatAction, InputFile, Message, PhotoSize},
    utils::command::BotCommands,
};

use crate::{
    commands::{Commands, Mode, Opt},
    resources::ensure_models,
};

mod commands;
mod resources;

fn best_photos(photos: &[PhotoSize]) -> Vec<&PhotoSize> {
    let mut unique_photos: HashMap<&str, &PhotoSize> = HashMap::new();
    for photo in photos {
        let id = photo.file.unique_id.as_str();
        if let Some(photo_in_map) = unique_photos.get(id) {
            if photo.height > photo_in_map.height {
                unique_photos.insert(id, photo);
            }
        } else {
            unique_photos.insert(id, photo);
        }
    }
    unique_photos.into_values().collect()
}

fn image_from_message(message: &Message) -> Option<&str> {
    message
        .document()
        .and_then(|doc| {
            doc.mime_type.as_ref().and_then(|mime| {
                if mime.type_() == mime::IMAGE {
                    Some(doc.file.id.as_str())
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            message
                .photo()
                .and_then(|photos| Some(best_photos(photos).first()?.file.id.as_str()))
        })
        .or_else(|| {
            message
                .sticker()
                .and_then(|sticker| (!sticker.is_animated()).then_some(sticker.file.id.as_str()))
        })
}

fn decode_image(bytes: &[u8]) -> Result<DynamicImage> {
    let guessed_image = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    if guessed_image.format() == Some(ImageFormat::WebP) {
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

async fn answer(bot: Bot, msg: Message, command: Commands) -> ResponseResult<()> {
    match command {
        Commands::Help | Commands::Start => {
            bot.send_message(msg.chat.id, Opt::command().render_long_help().to_string())
                .await?
        }
        Commands::Ciyaify(opt) => match opt {
            Err(err) => bot.send_message(msg.chat.id, err.to_string()).await?,
            Ok(opt) => {
                let message = msg.reply_to_message();
                match message {
                    None => {
                        bot.send_message(
                            msg.chat.id,
                            "Please reply to the image you want to ciyaify.",
                        )
                        .await?
                    }
                    Some(message) => {
                        let file_id = image_from_message(message);
                        match file_id {
                            None => {
                                bot.send_message(
                                    msg.chat.id,
                                    "Please reply to the image you want to ciyaify.",
                                )
                                .await?
                            }
                            Some(file_id) => {
                                if opt.antialias_scale > 8 {
                                    bot.send_message(msg.chat.id, "antialias_scale must <= 8.")
                                        .await?
                                } else {
                                    #[allow(unused_must_use)]
                                    {
                                        bot.send_chat_action(msg.chat.id, ChatAction::Typing).await;
                                    };

                                    let mut buffer = Vec::new();
                                    bot.download_file(
                                        &bot.get_file(file_id).await?.path,
                                        &mut buffer,
                                    )
                                    .await?;
                                    info!("Downloading model");
                                    let models = ensure_models();
                                    info!("Model downloaded");
                                    match models {
                                        None => {
                                            bot.send_message(msg.chat.id, "Unable to load model.")
                                                .await?
                                        }
                                        Some((face_model, landmark_model)) => {
                                            match decode_image(&buffer) {
                                                Err(_) => {
                                                    bot.send_message(
                                                        msg.chat.id,
                                                        "Invalid image format.",
                                                    )
                                                    .await?
                                                }
                                                Ok(image) => {
                                                    let output = {
                                                        let detector = match opt.mode {
                                                            Mode::Weeb => Box::new(
                                                                match WeebDetector::new(
                                                                    face_model.to_str().unwrap(),
                                                                    landmark_model
                                                                        .to_str()
                                                                        .unwrap(),
                                                                ) {
                                                                    Ok(detector) => detector,
                                                                    Err(e) => {
                                                                        warn!(
                                                                            "Unable to load \
                                                                             model: {}",
                                                                            e
                                                                        );
                                                                        return Ok(());
                                                                    }
                                                                },
                                                            ),
                                                            Mode::Standard => {
                                                                bot.send_message(
                                                                    msg.chat.id,
                                                                    "Standard detector not \
                                                                     implemented.",
                                                                )
                                                                .await?;
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
                                                            bot.send_message(
                                                                msg.chat.id,
                                                                "No face or mouth detected.",
                                                            )
                                                            .await?
                                                        }
                                                        Err(err) => {
                                                            bot.send_message(
                                                                msg.chat.id,
                                                                format!("{}", err),
                                                            )
                                                            .await?
                                                        }
                                                        Ok(output) => {
                                                            let encoder =
                                                                webp::Encoder::from_image(&output)
                                                                    .unwrap();
                                                            let bytes: Vec<u8> =
                                                                (*encoder.encode(80.)).to_vec();
                                                            bot.send_document(
                                                                msg.chat.id,
                                                                InputFile::memory(bytes)
                                                                    .file_name("ciya.webp"),
                                                            )
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
    pretty_env_logger::init();
    info!("Starting ciya_bot...");

    let bot = Bot::from_env();

    teloxide::commands_repl(bot, answer, Commands::ty()).await;
}
