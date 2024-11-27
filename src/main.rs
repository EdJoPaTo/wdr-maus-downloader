use std::fmt::Write;
use std::time::{Duration, Instant};

use anyhow::Context;
use retry::retry;

use crate::downloaded::Downloaded;
use crate::image::{download_jpg, resize_to_tg_thumbnail};
use crate::scrape::Scraperesult;
use crate::telegram::Telegram;

mod daily;
mod downloaded;
mod ffmpeg;
mod image;
mod scrape;
mod telegram;
mod temporary;
mod wdr_media;

fn main() {
    let tg = Telegram::new();

    #[allow(clippy::never_loop)]
    loop {
        // Do not create load right on startup
        #[cfg(not(debug_assertions))]
        std::thread::sleep(Duration::from_secs(5 * 60)); // 5 min

        if let Err(err) = iteration(&tg) {
            println!("Iteration failed {err:#}");
            tg.send_err(&format!("ERROR {err:#}"));
        }

        #[cfg(debug_assertions)]
        break;
    }
}

fn iteration(tg: &Telegram) -> anyhow::Result<()> {
    use crate::daily::{Daily, Job};
    let mut daily = Daily::new();
    if let Some(job) = daily.get_next() {
        println!("\n\ndo {job:?}…");
        let downloaded = Downloaded::new();
        let all = match job {
            Job::AktuelleSunday | Job::AktuelleCheckup => scrape::get_aktuell(),
            Job::SachgeschichteMorning | Job::SachgeschichteEvening => {
                scrape::get_sachgeschichten()
            }
        }
        .context("Failed to scrape")?;
        println!("found {} videos", all.len());
        for scraperesult in all {
            if !downloaded.was_downloaded(&scraperesult.media)
                && !daily.is_error(&scraperesult.media)
            {
                match handle_one(tg, &scraperesult).context("Failed to download") {
                    Ok(()) => Downloaded::mark_downloaded(scraperesult.media),
                    Err(err) => {
                        daily.mark_error(scraperesult.media);
                        return Err(err);
                    }
                }
                if !matches!(job, Job::AktuelleCheckup | Job::AktuelleSunday) {
                    break;
                }
            }
        }
        daily.mark_successful(job);
    }
    Ok(())
}

fn handle_one(tg: &Telegram, video: &Scraperesult) -> anyhow::Result<()> {
    let topic = video.topic;
    let img = &video.img;
    let media = &video.media;
    let title = &media.tracker_data.title;
    let air_time = &media.tracker_data.air_time;
    let video = media.media_resource.get_video();
    let sl = media.media_resource.get_sl_video();
    let mut caption_srt = media.media_resource.captions_hash.srt.as_ref();
    println!(
        "found {topic} to download {air_time} {title}\nImage: {}\nVideo: {}\nSign Language: {:?}\nCaptions: {:?}",
        img.as_str(),
        video.as_str(),
        sl.map(url::Url::as_str),
        caption_srt.map(url::Url::as_str)
    );

    if caption_srt.is_some_and(|url| url.path().ends_with("deleted")) {
        println!("Ignore caption as it ends with 'deleted'.");
        caption_srt = None;
    }

    let public_caption = format!("{title}\n{air_time} #{topic}");
    let meta_msg = tg.send_begin(img, &public_caption)?;

    let start = Instant::now();
    let thumbnail = {
        let img_downloaded = download_jpg(img)?;
        resize_to_tg_thumbnail(img_downloaded.path())?
    };
    let thumbnail_took = start.elapsed();
    let thumbnail_filesize =
        path_filesize_string(thumbnail.path()).expect("cant read thumbnail size");
    println!(
        "thumbnail took {}  {thumbnail_filesize} / 200 kB",
        format_duration(thumbnail_took)
    );

    let start = Instant::now();
    let normal = ffmpeg::download(video, caption_srt)?;
    let sl = if let Some(sl) = &sl {
        Some(ffmpeg::download(sl, caption_srt)?)
    } else {
        None
    };
    let download_took = start.elapsed();
    println!("download took {}", format_duration(download_took));

    let normal_filesize = path_filesize_string(normal.path()).expect("cant read video size");
    let sl_filesize = sl.as_ref().map_or_else(
        || "nope :(".into(),
        |sl| path_filesize_string(sl.path()).expect("cant read video size"),
    );
    println!("Filesizes   Normal: {normal_filesize}   DGS: {sl_filesize}");

    let mut meta_caption = format!(
        "{public_caption}\n\nThumbnail: {thumbnail_filesize} / 200 kB\nNormal: {normal_filesize}\nDGS: {sl_filesize}\n\ndownload took {}\n",
        format_duration(download_took)
    );
    retry(retry::delay::Fixed::from_millis(60_000).take(2), || {
        tg.update_meta(meta_msg, &meta_caption)
    })
    .map_err(anyhow::Error::msg)?;

    let start = Instant::now();
    tg.send_public_result(
        &public_caption,
        thumbnail.path().to_path_buf(),
        normal.path().to_path_buf(),
        sl.as_ref().map(|tempfile| tempfile.path().to_path_buf()),
    )?;
    let upload_took = start.elapsed();
    println!("upload   took {}", format_duration(upload_took));

    writeln!(meta_caption, "upload took {}", format_duration(upload_took)).unwrap();
    retry(retry::delay::Fixed::from_millis(60_000).take(2), || {
        tg.update_meta(meta_msg, &meta_caption)
    })
    .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn path_filesize_string(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(format_filesize(path.metadata()?.len()))
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    let seconds = total_seconds % 60.0;
    let minutes = (total_seconds / 60.0).floor();
    format!("{minutes:.0} min {seconds:.2} sec")
}

#[test]
fn format_duration_works() {
    assert_eq!("0 min 42.00 sec", format_duration(Duration::from_secs(42)));
    assert_eq!("1 min 22.00 sec", format_duration(Duration::from_secs(82)));
}

#[allow(clippy::cast_precision_loss)]
fn format_filesize(size: u64) -> String {
    let size = size as f32;
    let kb = size / 1024.0;
    if kb < 990.0 {
        return format!("{kb:3.1}kB");
    }
    let mb = kb / 1024.0;
    format!("{mb:3.1}MB")
}

#[test]
fn format_filesize_works() {
    assert_eq!("0.0kB", format_filesize(0));
    assert_eq!("12.1kB", format_filesize(12_345));
    assert_eq!("120.6kB", format_filesize(123_456));
    assert_eq!("117.7MB", format_filesize(123_456_789));
}
