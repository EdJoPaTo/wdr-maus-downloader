use std::sync::LazyLock;

use anyhow::Context;
use lazy_regex::regex;
use scraper::{ElementRef, Selector};
use url::Url;

use crate::wdr_media::WdrMedia;

#[derive(Debug, Clone, Copy)]
pub enum Topic {
    AktuelleSendung,
    Sachgeschichte,
    Zukunft,
}

impl core::fmt::Display for Topic {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, fmt)
    }
}

pub struct Scraperesult {
    pub topic: Topic,
    pub img: Url,
    pub media: WdrMedia,
}

fn get(url: &str) -> anyhow::Result<String> {
    let body = ureq::get(url).call()?.into_string()?;
    #[cfg(not(debug_assertions))]
    std::thread::sleep(std::time::Duration::from_millis(250));
    Ok(body)
}

pub fn get_aktuell() -> anyhow::Result<Vec<Scraperesult>> {
    static AKTUELLE: LazyLock<Url> =
        LazyLock::new(|| Url::parse("https://www.wdrmaus.de/aktuelle-sendung/").unwrap());
    get_from_page(Topic::AktuelleSendung, &AKTUELLE)
}

pub fn get_sachgeschichten() -> anyhow::Result<Vec<Scraperesult>> {
    static SACHGESCHICHTEN: LazyLock<Url> = LazyLock::new(|| {
        Url::parse("https://www.wdrmaus.de/filme/sachgeschichten/index.php5?filter=alle").unwrap()
    });
    static ZUKUNFT: LazyLock<Url> =
        LazyLock::new(|| Url::parse("https://www.wdrmaus.de/extras/mausthemen/zukunft/").unwrap());

    let mut videos = Vec::new();
    videos.append(&mut get_linked_videos(Topic::Zukunft, &ZUKUNFT)?);
    videos.append(&mut get_linked_videos(
        Topic::Sachgeschichte,
        &SACHGESCHICHTEN,
    )?);
    Ok(videos)
}

fn get_linked_videos(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    static LINK: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".links a").unwrap());

    let body = get(base.as_ref()).context("LinkedVideos")?;
    let body = scraper::Html::parse_document(&body);
    let links = body
        .select(&LINK)
        .filter_map(|elem| elem.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .collect::<Vec<_>>();
    let videos = get_many_pages(topic, &links);
    anyhow::ensure!(!videos.is_empty(), "no linked videos");
    Ok(videos)
}

fn get_many_pages(topic: Topic, links: &[Url]) -> Vec<Scraperesult> {
    let total = links.len();
    let mut videos = Vec::new();
    for link in links {
        match get_from_page(topic, link) {
            Ok(mut vec) => {
                videos.append(&mut vec);
                if videos.len() % 25 == 0 {
                    println!("{:>4}/{total:<4} {topic}", videos.len());
                }
            }
            Err(err) => println!("{topic} scrape {link} failed: {err:#}"),
        };
    }
    videos
}

fn get_from_page(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    fn from_container(base: &Url, videocontainer: ElementRef) -> anyhow::Result<(Url, WdrMedia)> {
        static IMG: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img").unwrap());

        let img = videocontainer
            .select(&IMG)
            .find_map(|elem| elem.value().attr("src"))
            .context("img not found")?;
        let img = base.join(img)?;

        let inner_html = videocontainer.inner_html();
        let media_object_url = regex!(r#"https?:[^'"]+\d+\.(?:js|assetjsonp)"#)
            .find(&inner_html)
            .context("media object url not found")?
            .as_str();
        let media = get(media_object_url)?;
        let begin = media.find('{').unwrap_or_default();
        #[allow(clippy::string_slice)]
        let media = media[begin..].trim_end_matches(&[')', ';']);
        let media = serde_json::from_str::<WdrMedia>(media)?;
        Ok((img, media))
    }

    static VIDEOCONTAINER: LazyLock<Selector> =
        LazyLock::new(|| Selector::parse(".videocontainer, .item.video").unwrap());

    let body = get(base.as_str())?;
    let body = scraper::Html::parse_document(&body);

    let mut videos = Vec::new();
    let containers = body.select(&VIDEOCONTAINER);
    for container in containers {
        let (img, media) = from_container(base, container)?;
        videos.push(Scraperesult { topic, img, media });
    }
    match videos.len() {
        0 => anyhow::bail!("no videos"),
        1 => {} // expected default
        many => println!("page has {many} videos"),
    }
    Ok(videos)
}
