use std::env;

#[cfg(feature = "webhook")]
use teloxide::dispatching::update_listeners::UpdateListener;
#[cfg(feature = "webhook")]
use teloxide::prelude::*;

#[cfg(feature = "webhook")]
pub mod webhook;

pub enum Listener {
    Polling,
    #[cfg(feature = "webhook")]
    Webhook(webhook::HTTPConfig),
}

impl Listener {
    pub fn from_env() -> Self {
        if let (Ok(base), Ok(path), Ok(addr)) = (
            env::var("APP_WEBHOOK_URL"),
            env::var("APP_WEBHOOK_PATH"),
            env::var("APP_BIND_ADDR"),
        ) {
            #[cfg(not(feature = "webhook"))]
            panic!("webhook support not enabled");
            #[cfg(feature = "webhook")]
            Self::Webhook(webhook::HTTPConfig::new(
                base.as_str(),
                path.as_str(),
                addr.as_str(),
            ))
        } else {
            Self::Polling
        }
    }

    #[cfg(feature = "webhook")]
    pub async fn try_into_webhook(
        self,
        bot: AutoSend<Bot>,
    ) -> impl UpdateListener<serde_json::Error> {
        if let Listener::Webhook(config) = self {
            webhook::listener(bot, config).await
        } else {
            panic!("not a webhook listener")
        }
    }
}
