use std::path::PathBuf;

use frankenstein::{
    Api, InputMediaPhoto, InputMediaVideo, Media, SendMediaGroupParams, SendMessageParams,
    SendPhotoParams, TelegramApi,
};
use url::Url;

#[cfg(not(debug_assertions))]
const PUBLIC_CHANNEL: i64 = -1_001_155_474_248;
#[cfg(not(debug_assertions))]
const META_CHANNEL: i64 = -1_001_214_301_516;

#[cfg(debug_assertions)]
const PUBLIC_CHANNEL: i64 = -1_001_149_205_144;
#[cfg(debug_assertions)]
const META_CHANNEL: i64 = -1_001_149_205_144;

pub struct Telegram {
    api: Api,
}

impl Telegram {
    pub fn new() -> Self {
        let bot_token = std::env::var("BOT_TOKEN").expect("set BOT_TOKEN via environment variable");

        let api = if let Ok(api_root) = std::env::var("TELEGRAM_API_ROOT") {
            println!("Telegram Bot custom api endpoint: {}", api_root);
            Api::new_url(format!("{}/bot{}", api_root, bot_token))
        } else {
            println!("Telegram Bot uses official api");
            Api::new(&bot_token)
        };

        let me = api.get_me().expect("Telegram get_me failed");
        println!(
            "Telegram acts as @{}",
            me.result.username.expect("Bot has no username")
        );

        Self { api }
    }

    pub fn send_err(&self, text: &str) -> anyhow::Result<()> {
        self.api.send_message(
            &SendMessageParams::builder()
                .chat_id(META_CHANNEL)
                .text(text)
                .build(),
        )?;
        Ok(())
    }

    pub fn send_begin(&self, img: &Url, text: &str) -> anyhow::Result<i32> {
        let message_id = self
            .api
            .send_photo(
                &SendPhotoParams::builder()
                    .chat_id(META_CHANNEL)
                    .caption(text)
                    .photo(img.to_string())
                    .build(),
            )?
            .result
            .message_id;
        Ok(message_id)
    }

    pub fn send_done(&self, reply_to: i32, text: &str) -> anyhow::Result<()> {
        self.api.send_message(
            &SendMessageParams::builder()
                .chat_id(META_CHANNEL)
                .text(text)
                .reply_to_message_id(reply_to)
                .build(),
        )?;
        Ok(())
    }

    pub fn send_public_result(
        &self,
        caption: &str,
        img: &Url,
        normal: PathBuf,
        sl: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        let mut media = vec![
            Media::Photo(
                InputMediaPhoto::builder()
                    .caption(caption)
                    .media(img.to_string())
                    .build(),
            ),
            Media::Video(InputMediaVideo::builder().media(normal).build()),
        ];
        if let Some(sl) = sl {
            media.push(Media::Video(InputMediaVideo::builder().media(sl).build()));
        }
        self.api.send_media_group(
            &SendMediaGroupParams::builder()
                .chat_id(PUBLIC_CHANNEL)
                .media(media)
                .build(),
        )?;
        Ok(())
    }
}
