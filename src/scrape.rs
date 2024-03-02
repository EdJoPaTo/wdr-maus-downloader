use lazy_regex::regex;
use once_cell::sync::Lazy;
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
    static AKTUELLE: Lazy<Url> =
        Lazy::new(|| Url::parse("https://www.wdrmaus.de/aktuelle-sendung/").unwrap());
    get_from_page(Topic::AktuelleSendung, &AKTUELLE)
}

pub fn get_sachgeschichten() -> anyhow::Result<Vec<Scraperesult>> {
    static SACHGESCHICHTEN: Lazy<Url> = Lazy::new(|| {
        Url::parse("https://www.wdrmaus.de/filme/sachgeschichten/index.php5?filter=alle").unwrap()
    });
    static ZUKUNFT: Lazy<Url> =
        Lazy::new(|| Url::parse("https://www.wdrmaus.de/extras/mausthemen/zukunft/").unwrap());

    let mut videos = Vec::new();
    videos.append(&mut get_linked_videos(Topic::Zukunft, &ZUKUNFT)?);
    videos.append(&mut get_linked_videos(
        Topic::Sachgeschichte,
        &SACHGESCHICHTEN,
    )?);
    Ok(videos)
}

fn get_linked_videos(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    static LINK: Lazy<Selector> = Lazy::new(|| Selector::parse(".links a").unwrap());

    let body = get(base.as_ref()).map_err(|err| anyhow::anyhow!("LinkedVideos: {err}"))?;
    let body = scraper::Html::parse_document(&body);
    let links = body
        .select(&LINK)
        .filter_map(|o| o.value().attr("href"))
        .filter_map(|o| base.join(o).ok())
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
            Ok(mut v) => {
                videos.append(&mut v);
                if videos.len() % 25 == 0 {
                    println!("{:>4}/{total:<4} {topic}", videos.len());
                }
            }
            Err(err) => println!("{topic} {err} {link}"),
        };
    }
    videos
}

fn get_from_page(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    fn from_container(base: &Url, videocontainer: ElementRef) -> anyhow::Result<(Url, WdrMedia)> {
        static IMG: Lazy<Selector> = Lazy::new(|| Selector::parse("img").unwrap());

        let img = videocontainer
            .select(&IMG)
            .find_map(|e| e.value().attr("src"))
            .ok_or_else(|| anyhow::anyhow!("img not found"))?;
        let img = base.join(img)?;

        let inner_html = videocontainer.inner_html();
        let media_object_url = regex!(r#"https?:[^'"]+\d+\.(?:js|assetjsonp)"#)
            .find(&inner_html)
            .ok_or_else(|| anyhow::anyhow!("media object url not found"))?
            .as_str();
        let media = get(media_object_url)?;
        let begin = media.find('{').unwrap_or_default();
        let media = media[begin..].trim_end_matches(&[')', ';']);
        let media = serde_json::from_str::<WdrMedia>(media)?;
        Ok((img, media))
    }

    static VIDEOCONTAINER: Lazy<Selector> =
        Lazy::new(|| Selector::parse(".videocontainer, .item.video").unwrap());

    let body = get(base.as_str())?;
    let body = scraper::Html::parse_document(&body);

    let mut videos = Vec::new();
    let containers = body.select(&VIDEOCONTAINER);
    for container in containers {
        let (img, media) = from_container(base, container)?;
        videos.push(Scraperesult { topic, img, media });
    }
    match videos.len() {
        0 => anyhow::bail!("no videos on {}", base.as_str()),
        1 => {} // expected default
        many => println!("page has {many} videos {}", base.as_str()),
    }
    Ok(videos)
}
