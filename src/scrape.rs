use std::sync::LazyLock;

use anyhow::Context as _;
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

fn get(url: &str) -> anyhow::Result<String> {
    let body = ureq::get(url).call()?.into_body().read_to_string()?;
    #[cfg(not(debug_assertions))]
    std::thread::sleep(std::time::Duration::from_millis(250));
    Ok(body)
}

pub struct Scraperesult {
    pub topic: Topic,
    pub img: Url,
    pub media: WdrMedia,
}

pub struct Scrape {
    links: Vec<(Topic, Url)>,
}

impl Scrape {
    pub fn get_aktuell() -> Self {
        static AKTUELLE: LazyLock<Url> =
            LazyLock::new(|| Url::parse("https://www.wdrmaus.de/aktuelle-sendung/").unwrap());
        Self {
            links: vec![(Topic::AktuelleSendung, AKTUELLE.clone())],
        }
    }

    pub fn get_sachgeschichten() -> anyhow::Result<Self> {
        static SACHGESCHICHTEN: LazyLock<Url> = LazyLock::new(|| {
            Url::parse("https://www.wdrmaus.de/filme/sachgeschichten/index.php5?filter=alle")
                .unwrap()
        });
        static ZUKUNFT: LazyLock<Url> = LazyLock::new(|| {
            Url::parse("https://www.wdrmaus.de/extras/mausthemen/zukunft/").unwrap()
        });

        let mut links = Vec::new();
        links.append(&mut Self::get_linked(Topic::Sachgeschichte, &SACHGESCHICHTEN)?.links);
        links.append(&mut Self::get_linked(Topic::Zukunft, &ZUKUNFT)?.links);
        Ok(Self { links })
    }

    fn get_linked(topic: Topic, base: &Url) -> anyhow::Result<Self> {
        static LINK: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".links a").unwrap());

        let body = get(base.as_ref()).context("LinkedVideos")?;
        let body = scraper::Html::parse_document(&body);
        let links = body
            .select(&LINK)
            .filter_map(|elem| elem.value().attr("href"))
            .filter_map(|href| base.join(href).ok())
            .map(|url| (topic, url))
            .rev() // Vec::pop starts at the end
            .collect::<Vec<_>>();
        anyhow::ensure!(!links.is_empty(), "no linked video pages");
        Ok(Self { links })
    }

    pub const fn len(&self) -> usize {
        self.links.len()
    }
}

impl Iterator for Scrape {
    type Item = anyhow::Result<Vec<Scraperesult>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (topic, link) = self.links.pop()?;
        let scraperesult =
            get_from_page(topic, &link).with_context(|| format!("{topic} scrape {link} failed"));
        Some(scraperesult)
    }
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
        #[expect(clippy::string_slice)]
        let media = media[begin..].trim_end_matches([')', ';']);
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
