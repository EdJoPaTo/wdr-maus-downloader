use std::path::PathBuf;

use frankenstein::{
    Api, EditMessageCaptionParams, InputMediaVideo, Media, SendMediaGroupParams, SendMessageParams,
    SendPhotoParams, SendVideoParams, TelegramApi,
};
use url::Url;

use crate::ffmpeg::{extract_video_thumbnail, VideoStats};
use crate::image::resize_to_tg_thumbnail;

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

        let api = std::env::var("TELEGRAM_API_ROOT").map_or_else(
            |_| {
                println!("Telegram Bot uses official api");
                Api::new(&bot_token)
            },
            |api_root| {
                println!("Telegram Bot custom api endpoint: {api_root}");
                Api::new_url(format!("{api_root}/bot{bot_token}"))
            },
        );

        let me = api.get_me().expect("Telegram get_me failed");
        println!(
            "Telegram acts as @{}",
            me.result.username.expect("Bot has no username")
        );

        Self { api }
    }

    pub fn send_err(&self, text: &str) {
        self.api
            .send_message(
                &SendMessageParams::builder()
                    .chat_id(META_CHANNEL)
                    .text(text)
                    .build(),
            )
            .expect("Send error to Telegram failed");
    }

    pub fn send_begin(&self, img: &Url, text: &str) -> anyhow::Result<i32> {
        let message_id = self
            .api
            .send_photo(
                &SendPhotoParams::builder()
                    .chat_id(META_CHANNEL)
                    .disable_notification(true)
                    .caption(text)
                    .photo(img.to_string())
                    .build(),
            )
            .map_err(|err| anyhow::anyhow!("Telegram::send_photo {err}"))?
            .result
            .message_id;
        Ok(message_id)
    }

    pub fn update_meta(&self, msg_id: i32, text: &str) -> anyhow::Result<()> {
        self.api
            .edit_message_caption(
                &EditMessageCaptionParams::builder()
                    .chat_id(META_CHANNEL)
                    .message_id(msg_id)
                    .caption(text)
                    .build(),
            )
            .map_err(|err| anyhow::anyhow!("Telegram::edit_message_caption {err}"))?;
        Ok(())
    }

    pub fn send_public_result(
        &self,
        caption: &str,
        thumbnail: PathBuf,
        normal: PathBuf,
        sl: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        if let Some(sl) = sl {
            let sl_big_thumbnail = extract_video_thumbnail(sl.as_path())?;
            let sl_tg_thumbnail = resize_to_tg_thumbnail(sl_big_thumbnail.path())?;
            let media = vec![
                build_media_group_video(normal, caption, thumbnail)?,
                build_media_group_video(sl, "", sl_tg_thumbnail.path().to_path_buf())?,
            ];
            self.api
                .send_media_group(
                    &SendMediaGroupParams::builder()
                        .chat_id(PUBLIC_CHANNEL)
                        .media(media)
                        .build(),
                )
                .map_err(|err| anyhow::anyhow!("Telegram::send_media_group {err}"))?;
        } else {
            let stats = VideoStats::load(&normal)?;
            self.api
                .send_video(
                    &SendVideoParams::builder()
                        .supports_streaming(true)
                        .chat_id(PUBLIC_CHANNEL)
                        .video(normal)
                        .caption(caption)
                        .thumbnail(thumbnail)
                        .duration(stats.duration)
                        .width(stats.width)
                        .height(stats.height)
                        .build(),
                )
                .map_err(|err| anyhow::anyhow!("Telegram::send_video {err}"))?;
        }
        Ok(())
    }
}

fn build_media_group_video(
    media: PathBuf,
    caption: &str,
    thumbnail: PathBuf,
) -> anyhow::Result<Media> {
    let stats = VideoStats::load(&media)?;
    let video = InputMediaVideo::builder()
        .supports_streaming(true)
        .media(media)
        .caption(caption)
        .thumbnail(thumbnail)
        .duration(stats.duration)
        .width(stats.width)
        .height(stats.height)
        .build();
    Ok(Media::Video(video))
}
