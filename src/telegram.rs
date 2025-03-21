use std::path::Path;

use anyhow::Context as _;
use frankenstein::TelegramApi as _;
use frankenstein::client_ureq::Bot;
use frankenstein::input_media::{InputMediaVideo, MediaGroupInputMedia};
use frankenstein::methods::{
    EditMessageCaptionParams, SendMediaGroupParams, SendMessageParams, SendPhotoParams,
    SendVideoParams,
};
use url::Url;

use crate::ffmpeg::{VideoStats, extract_video_thumbnail};
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
    bot: Bot,
}

impl Telegram {
    pub fn new() -> Self {
        let bot_token = std::env::var("BOT_TOKEN").expect("set BOT_TOKEN via environment variable");

        let bot = std::env::var("TELEGRAM_API_ROOT").map_or_else(
            |_| {
                println!("Telegram Bot uses official api");
                Bot::new(&bot_token)
            },
            |api_root| {
                println!("Telegram Bot custom api endpoint: {api_root}");
                Bot::new_url(format!("{api_root}/bot{bot_token}"))
            },
        );

        let me = bot.get_me().expect("Telegram get_me failed");
        println!(
            "Telegram acts as @{}",
            me.result.username.expect("Bot has no username")
        );

        Self { bot }
    }

    pub fn send_err(&self, text: &str) {
        self.bot
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
            .bot
            .send_photo(
                &SendPhotoParams::builder()
                    .chat_id(META_CHANNEL)
                    .disable_notification(true)
                    .caption(text)
                    .photo(img.to_string())
                    .build(),
            )
            .context("Telegram::send_photo")?
            .result
            .message_id;
        Ok(message_id)
    }

    pub fn update_meta(&self, msg_id: i32, text: &str) -> anyhow::Result<()> {
        self.bot
            .edit_message_caption(
                &EditMessageCaptionParams::builder()
                    .chat_id(META_CHANNEL)
                    .message_id(msg_id)
                    .caption(text)
                    .build(),
            )
            .context("Telegram::edit_message_caption")?;
        Ok(())
    }

    pub fn send_public_result(
        &self,
        caption: &str,
        cover: &Path,
        thumbnail: &Path,
        normal: &Path,
        sl: Option<&Path>,
    ) -> anyhow::Result<()> {
        if let Some(sl) = sl {
            let sl_big_thumbnail = extract_video_thumbnail(sl)?;
            let sl_tg_thumbnail = resize_to_tg_thumbnail(sl_big_thumbnail.path())?;
            let media = vec![
                build_media_group_video(normal, caption, Some(cover), thumbnail)?,
                build_media_group_video(sl, "", None, sl_tg_thumbnail.path())?,
            ];
            self.bot
                .send_media_group(
                    &SendMediaGroupParams::builder()
                        .chat_id(PUBLIC_CHANNEL)
                        .media(media)
                        .build(),
                )
                .context("Telegram::send_media_group")?;
        } else {
            let stats = VideoStats::load(normal)?;
            self.bot
                .send_video(
                    &SendVideoParams::builder()
                        .supports_streaming(true)
                        .chat_id(PUBLIC_CHANNEL)
                        .video(normal.to_path_buf())
                        .caption(caption)
                        .cover(cover.to_path_buf())
                        .thumbnail(thumbnail.to_path_buf())
                        .duration(stats.duration)
                        .width(stats.width)
                        .height(stats.height)
                        .build(),
                )
                .context("Telegram::send_video")?;
        }
        Ok(())
    }
}

fn build_media_group_video(
    media: &Path,
    caption: &str,
    cover: Option<&Path>,
    thumbnail: &Path,
) -> anyhow::Result<MediaGroupInputMedia> {
    let stats = VideoStats::load(media)?;
    let video = InputMediaVideo::builder()
        .supports_streaming(true)
        .media(media.to_path_buf())
        .caption(caption)
        .maybe_cover(cover.map(Path::to_path_buf))
        .thumbnail(thumbnail.to_path_buf())
        .duration(stats.duration)
        .width(stats.width)
        .height(stats.height)
        .build();
    Ok(MediaGroupInputMedia::Video(video))
}
