use std::str::FromStr;

use ciya_lib::ciyafier::Emotion;
use clap::{ColorChoice, Parser, ValueEnum};
use teloxide::{macros::BotCommands, utils::command::ParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("shell-like parse error")]
    ShellParseError(#[from] shellwords::MismatchedQuotes),
    #[error("clap parse error")]
    ClapError(#[from] clap::Error),
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Mode {
    Weeb,
    Standard,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CliEmotion {
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
#[command(name = "ciya-bot")]
#[command(author, version, about)]
#[command(no_binary_name = true, color = ColorChoice::Never, disable_help_flag = true)]
pub struct Opt {
    #[arg(value_enum, default_value_t = CliEmotion::Auto)]
    pub emotion: CliEmotion,
    #[arg(value_enum, default_value_t = Mode::Weeb)]
    pub mode: Mode,
    #[arg(default_value_t = 8)]
    pub antialias_scale: u32,
}

impl FromStr for Opt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_string = shellwords::split(s)?;
        Ok(Self::try_parse_from(&split_string)?)
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn parse_opt(input: String) -> Result<(Result<Opt, String>,), ParseError> {
    Ok((Opt::from_str(&input).map_err(|e| e.to_string()),))
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Commands {
    #[command(
    description = "ciyaify an image.",
    parse_with = parse_opt
    )]
    Ciyaify(Result<Opt, String>),
    #[command(description = "help of this bot.")]
    Help,
    #[command(description = "help of this bot.")]
    Start,
}
