use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::scrape::Scraperesult;
use crate::telegram::Telegram;
use crate::wdr_media::WdrMedia;

mod ffmpeg;
mod scrape;
mod telegram;
mod wdr_media;

const DOWNLOADED_PATH: &str = "downloaded.yaml";

#[cfg(not(debug_assertions))]
const EVERY_MINUTES: u8 = 10;

fn main() {
    let tg = Telegram::new();

    #[cfg(debug_assertions)]
    {
        do_sunday(&tg).unwrap();

        loop {
            do_evening(&tg).unwrap();
            println!("sleep…\n\n");
            sleep(Duration::from_secs(30));
        }
    }

    #[cfg(not(debug_assertions))]
    loop {
        const SLEEPTIME: Duration = Duration::from_secs(60 * (EVERY_MINUTES as u64));
        println!("sleep…\n\n");
        sleep(SLEEPTIME);

        if let Err(err) = iteration(&tg) {
            println!("Iteration failed {}", err);
        }
    }
}

#[cfg(not(debug_assertions))]
fn iteration(tg: &Telegram) -> anyhow::Result<()> {
    let now = time::OffsetDateTime::now_utc();
    if now.weekday() == time::Weekday::Sunday && now.hour() >= 7 && now.hour() <= 11 {
        do_sunday(tg)?;
    } else if now.hour() == 17 && now.minute() < EVERY_MINUTES {
        do_evening(tg)?;
    }
    Ok(())
}

fn do_sunday(tg: &Telegram) -> anyhow::Result<()> {
    println!("do sunday…");
    let all = scrape::get_aktuell()?;
    let downloaded = get_downloaded();
    for not_yet_downloaded in all.iter().filter(|o| !downloaded.contains(&o.media)) {
        match handle_one(tg, not_yet_downloaded) {
            Ok(_) => mark_downloaded(not_yet_downloaded.media.clone()),
            Err(err) => tg.send_err(&format!("{}", err))?,
        }
    }
    Ok(())
}

fn do_evening(tg: &Telegram) -> anyhow::Result<()> {
    println!("do evening…");
    let all = scrape::get_all()?;
    dbg!(all.len());
    let downloaded = get_downloaded();
    if let Some(not_yet_downloaded) = all.iter().find(|o| !downloaded.contains(&o.media)) {
        match handle_one(tg, not_yet_downloaded) {
            Ok(_) => mark_downloaded(not_yet_downloaded.media.clone()),
            Err(err) => tg.send_err(&format!("{}", err))?,
        }
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
    println!("found {} to download {}", topic, img.as_str());
    println!("{} {}", air_time, title);
    dbg!(
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
    println!("download took {:?}", download_took);

    let start = Instant::now();
    tg.send_public_result(
        &public_caption,
        img,
        normal.path().to_path_buf(),
        sl.as_ref().map(|o| o.path().to_path_buf()),
    )?;
    let upload_took = start.elapsed();
    println!("upload   took {:?}", upload_took);

    let meta_caption = format!(
        "download took {:?}\nupload took {:?}\n\nNormal: {}\nDGS: {}",
        download_took,
        upload_took,
        path_filesize_string(normal.path()).expect("cant read video size"),
        sl.map_or_else(
            || "nope :(".into(),
            |sl| path_filesize_string(sl.path()).expect("cant read video size")
        ),
    );
    tg.send_done(meta_msg, &meta_caption)?;
    Ok(())
}

fn get_downloaded() -> Vec<WdrMedia> {
    serde_yaml::from_str(
        &std::fs::read_to_string(DOWNLOADED_PATH).unwrap_or_else(|_| "---\n[]".into()),
    )
    .expect("downloaded.yaml format error")
}

fn mark_downloaded(media: WdrMedia) {
    let mut downloaded = get_downloaded();
    downloaded.push(media);
    downloaded.sort();
    let content = serde_yaml::to_string(&downloaded).unwrap();
    std::fs::write(DOWNLOADED_PATH, content).expect("failed to write downloaded.yaml");
}

fn path_filesize_string(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(format_filesize(path.metadata()?.len()))
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
