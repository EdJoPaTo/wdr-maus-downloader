use std::path::PathBuf;

use frankenstein::{
    Api, InputMediaPhotoBuilder, InputMediaVideoBuilder, Media, SendMediaGroupParamsBuilder,
    SendMessageParamsBuilder, SendPhotoParamsBuilder, TelegramApi,
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
        self.api
            .send_message(
                &SendMessageParamsBuilder::default()
                    .chat_id(META_CHANNEL)
                    .text(text)
                    .build()
                    .unwrap(),
            )
            .map_err(map_tg_error)?;
        Ok(())
    }

    pub fn send_begin(&self, img: &Url, text: &str) -> anyhow::Result<i32> {
        let message_id = self
            .api
            .send_photo(
                &SendPhotoParamsBuilder::default()
                    .chat_id(META_CHANNEL)
                    .caption(text)
                    .photo(img.to_string())
                    .build()
                    .unwrap(),
            )
            .map_err(map_tg_error)?
            .result
            .message_id;
        Ok(message_id)
    }

    pub fn send_done(&self, reply_to: i32, text: &str) -> anyhow::Result<()> {
        self.api
            .send_message(
                &SendMessageParamsBuilder::default()
                    .chat_id(META_CHANNEL)
                    .text(text)
                    .reply_to_message_id(reply_to)
                    .build()
                    .unwrap(),
            )
            .map_err(map_tg_error)?;
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
                InputMediaPhotoBuilder::default()
                    .caption(caption)
                    .media(img.to_string())
                    .build()
                    .unwrap(),
            ),
            Media::Video(
                InputMediaVideoBuilder::default()
                    .media(normal)
                    .build()
                    .unwrap(),
            ),
        ];
        if let Some(sl) = sl {
            media.push(Media::Video(
                InputMediaVideoBuilder::default().media(sl).build().unwrap(),
            ));
        }
        self.api
            .send_media_group(
                &SendMediaGroupParamsBuilder::default()
                    .chat_id(PUBLIC_CHANNEL)
                    .media(media)
                    .build()
                    .unwrap(),
            )
            .map_err(map_tg_error)?;
        Ok(())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn map_tg_error(err: frankenstein::Error) -> anyhow::Error {
    anyhow::anyhow!("tgerr {:?}", err)
}
