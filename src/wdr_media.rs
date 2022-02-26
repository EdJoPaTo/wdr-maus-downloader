use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct WdrMedia {
    pub tracker_data: TrackerData,
    pub media_resource: MediaResources,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TrackerData {
    #[serde(alias = "trackerClipId")]
    pub id: String,

    #[serde(alias = "trackerClipAirTime")]
    pub air_time: String,

    #[serde(alias = "trackerClipTitle")]
    pub title: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum MediaFormat {
    Hls,
    Mp4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct MediaResource {
    pub media_format: MediaFormat,
    #[serde(alias = "videoURL", deserialize_with = "deserialize_url")]
    pub video: Url,

    #[serde(
        default,
        alias = "slVideoURL",
        deserialize_with = "deserialize_opt_url",
        skip_serializing_if = "Option::is_none"
    )]
    /// Deutsche Gebärdensprache
    /// -> Sign Language?
    pub sl_video: Option<Url>,

    #[serde(
        default,
        alias = "adVideoURL",
        deserialize_with = "deserialize_opt_url",
        skip_serializing_if = "Option::is_none"
    )]
    /// Audiodeskription
    pub ad_video: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct MediaResources {
    #[serde(
        default,
        deserialize_with = "deserialize_opt_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub preview_image: Option<Url>,

    /// does that mean `default`?
    pub dflt: MediaResource,
    /// alternative
    pub alt: MediaResource,
    #[serde(default)]
    pub captions_hash: Captions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Captions {
    #[serde(
        default,
        deserialize_with = "deserialize_opt_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub srt: Option<Url>,
}

pub fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let url = if s.starts_with("//") {
        format!("https:{}", s)
    } else {
        s
    };
    Url::parse(&url).map_err(serde::de::Error::custom)
}

pub fn deserialize_opt_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let url = deserialize_url(deserializer)?;
    Ok(Some(url))
}

impl MediaResources {
    pub const fn score(&self) -> usize {
        let mut score = 0;
        if self.captions_hash.srt.is_some() {
            score += 1;
        }
        if self.dflt.sl_video.is_some() {
            score += 1;
        }
        score
    }
}

#[test]
fn corona_example() {
    let json = r#"{
    "mediaResource": {
        "alt": {
            "mediaFormat": "hls",
            "videoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/weltweit/fsk0/234/2346162/,2346162_31733428,2346162_31733429,2346162_31733430,2346162_31733426,2346162_31733431,2346162_31733427,.mp4.csmil/master.m3u8"
        },
        "captionsHash": {
        },
        "dflt": {
            "mediaFormat": "hls",
            "videoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/weltweit/fsk0/234/2346162/,2346162_31733428,2346162_31733429,2346162_31733430,2346162_31733426,2346162_31733431,2346162_31733427,.mp4.csmil/master.m3u8"
        },
        "thumbnailTrack": {
            "url": "//wdrmedien-a.akamaihd.net/medp/ondemand/weltweit/fsk0/234/2346162/2346162_31733433.vtt"
        }
    },
    "mediaType": "vod",
    "mediaVersion": "1.4.0",
    "trackerData": {
        "trackerClipAgfCategory": "Information",
        "trackerClipAirTime": "07.03.2021 00:00",
        "trackerClipCategory": "WDR",
        "trackerClipId": "mdb-2346162",
        "trackerClipIsTrailer": "0",
        "trackerClipIsWebOnly": "0",
        "trackerClipSubcategory": "Die Maus wird 50",
        "trackerClipTitle": "Was sind Mutationen?"
    }
}"#;
    let media = serde_json::from_str::<WdrMedia>(json).unwrap();
    dbg!(media);
    // todo!();
}

#[test]
fn sendung_example() {
    let json = r#"{
    "mediaResource": {
        "alt": {
            "adVideoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40253983,2580812_40253984,2580812_40253985,2580812_40253982,.mp4.csmil/master.m3u8",
            "mediaFormat": "hls",
            "slVideoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40253987,2580812_40253988,2580812_40253989,2580812_40253986,.mp4.csmil/master.m3u8",
            "videoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40254024,2580812_40254025,2580812_40254026,2580812_40254023,.mp4.csmil/master.m3u8"
        },
        "captionsHash": {
            "srt": "//wdrmedien-a.akamaihd.net/medp/ondemand/deChAt/fsk0/258/2580812/2580812_40254488.srt",
            "vtt": "//wdrmedien-a.akamaihd.net/medp/ondemand/deChAt/fsk0/258/2580812/2580812_40254489.vtt",
            "xml": "//wdrmedien-a.akamaihd.net/medp/ondemand/deChAt/fsk0/258/2580812/2580812_40254487.xml"
        },
        "dflt": {
            "adVideoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40253983,2580812_40253984,2580812_40253985,2580812_40253982,.mp4.csmil/master.m3u8",
            "mediaFormat": "hls",
            "slVideoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40253987,2580812_40253988,2580812_40253989,2580812_40253986,.mp4.csmil/master.m3u8",
            "videoURL": "//wdradaptiv-vh.akamaihd.net/i/medp/ondemand/deChAt/fsk0/258/2580812/,2580812_40254024,2580812_40254025,2580812_40254026,2580812_40254023,.mp4.csmil/master.m3u8"
        },
        "previewImage": "https://kinder.wdr.de/tv/die-sendung-mit-der-maus/startbild_maus_100~_v-%%FORMAT%%.jpg"
    },
    "mediaType": "vod",
    "mediaVersion": "1.4.0",
    "trackerData": {
        "trackerClipAgfCategory": "Information",
        "trackerClipAirTime": "21.11.2021 09:30",
        "trackerClipCategory": "Das Erste",
        "trackerClipId": "mdb-2580812",
        "trackerClipIsTrailer": "0",
        "trackerClipIsWebOnly": "0",
        "trackerClipMeFoId": "X004611689",
        "trackerClipSubcategory": "Die Sendung mit der Maus",
        "trackerClipTitle": "Die Sendung vom 21.11.2021"
    }
}"#;
    let media = serde_json::from_str::<WdrMedia>(json).unwrap();
    dbg!(media);
    // todo!();
}
