#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::downloaded::Downloaded;
use crate::scrape::Scraperesult;
use crate::telegram::Telegram;

mod downloaded;
mod ffmpeg;
mod scrape;
mod telegram;
mod wdr_media;

const EVERY_MINUTES: u32 = 20;

#[cfg(debug_assertions)]
const SLEEPTIME: Duration = Duration::from_secs(30);
#[cfg(not(debug_assertions))]
const SLEEPTIME: Duration = Duration::from_secs(60 * (EVERY_MINUTES as u64));

fn main() {
    let tg = Telegram::new();

    #[cfg(not(debug_assertions))]
    sleep(SLEEPTIME);

    do_aktuelle(&tg).expect("startup do_aktuelle failed");

    loop {
        sleep(SLEEPTIME);

        #[cfg(debug_assertions)]
        do_sachgeschichte(&tg).unwrap();

        #[cfg(not(debug_assertions))]
        if let Err(err) = iteration(&tg) {
            println!("Iteration failed {}", err);
            tg.send_err(&format!("ERROR {}", err))
                .expect("send error to Telegram failed");
        }
    }
}

fn iteration(tg: &Telegram) -> anyhow::Result<()> {
    use chrono::{Datelike, Local, Timelike, Weekday};

    let now = Local::now();
    println!(
        "check time… {:>2}:{:>02} {}",
        now.hour(),
        now.minute(),
        now.weekday()
    );
    if now.weekday() == Weekday::Sun && now.hour() >= 8 && now.hour() <= 13 {
        do_aktuelle(tg)?;
    } else if now.minute() < EVERY_MINUTES {
        match now.hour() {
            16 => do_sachgeschichte(tg)?,
            19 => do_aktuelle(tg)?,
            _ => {}
        }
    }
    Ok(())
}

fn do_aktuelle(tg: &Telegram) -> anyhow::Result<()> {
    println!("\n\ndo aktuelle…");
    let all = scrape::get_aktuell()?;
    let downloaded = Downloaded::new();
    let all = all.iter().filter(|o| !downloaded.was_downloaded(&o.media));
    for not_yet_downloaded in all {
        handle_one(tg, not_yet_downloaded)?;
        Downloaded::mark_downloaded(not_yet_downloaded.media.clone());
    }
    Ok(())
}

fn do_sachgeschichte(tg: &Telegram) -> anyhow::Result<()> {
    println!("\n\ndo sachgeschichten…");
    let all = scrape::get_sachgeschichten()?;
    println!("found {} videos", all.len());
    let downloaded = Downloaded::new();
    if let Some(not_yet_downloaded) = all.iter().find(|o| !downloaded.was_downloaded(&o.media)) {
        handle_one(tg, not_yet_downloaded)?;
        Downloaded::mark_downloaded(not_yet_downloaded.media.clone());
    }
    Ok(())
}

fn handle_one(tg: &Telegram, video: &Scraperesult) -> anyhow::Result<()> {
    let topic = video.topic;
    let img = &video.img;
    let media = &video.media;
    let title = &media.tracker_data.title;
    let air_time = &media.tracker_data.air_time;
    let video = &media.media_resource.dflt.video;
    let sl = media.media_resource.dflt.sl_video.as_ref();
    let caption_srt = media.media_resource.captions_hash.srt.as_ref();
    println!(
        "found {} to download {} {}\nImage: {}\nVideo: {}\nSign Language: {:?}\nCaptions: {:?}",
        topic,
        air_time,
        title,
        img.as_str(),
        video.as_str(),
        sl.map(url::Url::as_str),
        caption_srt.map(url::Url::as_str)
    );

    let public_caption = format!("{}\n{} #{}", title, air_time, topic,);
    let meta_msg = tg.send_begin(img, &public_caption)?;

    let start = Instant::now();
    let normal = ffmpeg::download(video, caption_srt)?;
    let sl = if let Some(sl) = &sl {
        Some(ffmpeg::download(sl, caption_srt)?)
    } else {
        None
    };
    let download_took = start.elapsed();
    println!("download took {}", format_duration(download_took));

    let start = Instant::now();
    tg.send_public_result(
        &public_caption,
        img,
        normal.path().to_path_buf(),
        sl.as_ref().map(|o| o.path().to_path_buf()),
    )?;
    let upload_took = start.elapsed();
    println!("upload   took {}", format_duration(upload_took));

    let meta_caption = format!(
        "download took {}\nupload took {}\n\nNormal: {}\nDGS: {}",
        format_duration(download_took),
        format_duration(upload_took),
        path_filesize_string(normal.path()).expect("cant read video size"),
        sl.map_or_else(
            || "nope :(".into(),
            |sl| path_filesize_string(sl.path()).expect("cant read video size")
        ),
    );
    tg.send_done(meta_msg, &meta_caption)?;
    Ok(())
}

fn path_filesize_string(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(format_filesize(path.metadata()?.len()))
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    let seconds = total_seconds % 60.0;
    let minutes = (total_seconds / 60.0).floor();
    format!("{:.0} min {:.2} sec", minutes, seconds)
}

#[test]
fn format_duration_works() {
    assert_eq!("0 min 42.00 sec", format_duration(Duration::from_secs(42)));
    assert_eq!("1 min 22.00 sec", format_duration(Duration::from_secs(82)));
}

#[allow(clippy::cast_precision_loss)]
fn format_filesize(size: u64) -> String {
    let mb = (size as f32) / (1024.0 * 1024.0);
    format!("{:3.1}MB", mb)
}

#[test]
fn format_filesize_works() {
    assert_eq!("0.0MB", format_filesize(0));
    assert_eq!("117.7MB", format_filesize(123_456_789));
}
