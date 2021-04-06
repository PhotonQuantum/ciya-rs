use std::str::FromStr;

use clap::arg_enum;
use structopt::StructOpt;
use teloxide::utils::command::{BotCommand, ParseError};
use thiserror::Error;

use ciya_lib::ciyafier::Emotion;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("shell-like parse error")]
    ShellParseError(#[from] shellwords::MismatchedQuotes),
    #[error("clap parse error")]
    ClapError(#[from] clap::Error),
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum Mode{
        Weeb,
        Standard
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum CliEmotion{
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
            CliEmotion::Cry => Emotion::Cry,
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
name = "ciya-bot",
about = "Ciyaify your image.",
author = "LightQuantum <self@lightquantum.me>",
version = "0.1.0",
setting = clap::AppSettings::NoBinaryName,
setting = clap::AppSettings::ColorNever,
setting = clap::AppSettings::DisableHelpFlags
)]
pub struct Opt {
    #[structopt(possible_values = & CliEmotion::variants(), case_insensitive = true, default_value = "auto")]
    pub emotion: CliEmotion,
    #[structopt(possible_values = & Mode::variants(), case_insensitive = true, default_value = "weeb")]
    pub mode: Mode,
    #[structopt(default_value = "8")]
    pub antialias_scale: u32,
}

impl FromStr for Opt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_string = shellwords::split(s)?;
        Ok(Opt::from_iter_safe(&split_string)?)
    }
}

fn parse_opt(input: String) -> Result<(Result<Opt, anyhow::Error>,), ParseError> {
    Ok((Opt::from_str(&input),))
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(
        rename = "lowercase",
        description = "ciyaify an image.",
        parse_with = "parse_opt"
    )]
    Ciyaify(Result<Opt, anyhow::Error>),
    #[command(rename = "lowercase", description = "help of this bot.")]
    Help,
    #[command(rename = "lowercase", description = "help of this bot.")]
    Start,
}
