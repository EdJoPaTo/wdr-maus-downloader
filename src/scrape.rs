use regex::Regex;
use scraper::{ElementRef, Selector};
use url::Url;

use crate::wdr_media::WdrMedia;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Topic {
    AktuelleSendung,
    Sachgeschichte,
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
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

lazy_static::lazy_static! {
    static ref AKTUELLE: Url = Url::parse("https://www.wdrmaus.de/aktuelle-sendung/").unwrap();
    static ref SACHGESCHICHTEN: Url = Url::parse("https://www.wdrmaus.de/filme/sachgeschichten/index.php5?filter=alle").unwrap();

    static ref VIDEOCONTAINER: Selector = Selector::parse(".videocontainer, .item.video").unwrap();
}

pub fn get_aktuell() -> anyhow::Result<Vec<Scraperesult>> {
    get_themen_videos(Topic::AktuelleSendung, &AKTUELLE)
}

pub fn get_sachgeschichten() -> anyhow::Result<Vec<Scraperesult>> {
    get_linked_videos(Topic::Sachgeschichte, &SACHGESCHICHTEN)
}

fn get_linked_videos(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    lazy_static::lazy_static! {
        static ref LINK: Selector = Selector::parse(".links .dynamicteaser a").unwrap();
    }
    let body = get(base.as_ref()).map_err(|err| anyhow::anyhow!("LinkedVideos: {}", err))?;
    let body = scraper::Html::parse_document(&body);
    let links = body
        .select(&LINK)
        .filter_map(|o| o.value().attr("href"))
        .filter_map(|o| base.join(o).ok())
        .collect::<Vec<_>>();
    let mut videos = Vec::new();
    for link in &links {
        match get_themen_videos(topic, link) {
            Ok(mut v) => {
                videos.append(&mut v);
                if videos.len() % 25 == 0 {
                    println!("{:>4}/{:<4} {}", videos.len(), links.len(), topic);
                }
            }
            Err(err) => println!("{} {}", err, link),
        };
    }
    if videos.is_empty() {
        anyhow::bail!("no linked videos");
    }
    Ok(videos)
}

fn get_themen_videos(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
    fn inner(topic: Topic, base: &Url) -> anyhow::Result<Vec<Scraperesult>> {
        let body = get(base.as_str())?;
        let body = scraper::Html::parse_document(&body);
        let mut videos = Vec::new();
        let containers = body.select(&VIDEOCONTAINER);
        for container in containers {
            let (img, media) = get_video(base, container)?;
            videos.push(Scraperesult { topic, img, media });
        }
        if videos.is_empty() {
            anyhow::bail!("no videos on {}", base.as_str());
        } else if videos.len() > 1 {
            println!("page has {} videos {}", videos.len(), base.as_str());
        }
        Ok(videos)
    }
    inner(topic, base).map_err(|err| anyhow::anyhow!("{}: {}", topic, err))
}

fn get_video(base: &Url, videocontainer: ElementRef) -> anyhow::Result<(Url, WdrMedia)> {
    lazy_static::lazy_static! {
        static ref IMG: Selector = Selector::parse("img").unwrap();
        static ref MEDIA_OBJECT: Regex = Regex::new(r#"https?:[^'"]+\d+\.(?:js|assetjsonp)"#).unwrap();
    }

    let img = videocontainer
        .select(&IMG)
        .find_map(|e| e.value().attr("src"))
        .ok_or_else(|| anyhow::anyhow!("img not found"))?;
    let img = base.join(img)?;

    let inner_html = videocontainer.inner_html();
    let media_object_url = MEDIA_OBJECT
        .find(&inner_html)
        .ok_or_else(|| anyhow::anyhow!("media object url not found"))?
        .as_str();
    let media = get(media_object_url)?;
    let begin = media.find('{').unwrap_or_default();
    let media = media[begin..].trim_end_matches(&[')', ';']);
    let media = serde_json::from_str::<WdrMedia>(media)?;
    Ok((img, media))
}
