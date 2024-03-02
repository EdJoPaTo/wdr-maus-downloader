use crate::wdr_media::WdrMedia;

const DOWNLOADED_PATH: &str = "downloaded.yaml";

pub struct Downloaded {
    list: Vec<WdrMedia>,
}

impl Downloaded {
    #[must_use]
    pub fn new() -> Self {
        let list = std::fs::read_to_string(DOWNLOADED_PATH)
            .map(|content| serde_yaml::from_str(&content).expect("downloaded.yaml format error"))
            .unwrap_or_default();
        Self { list }
    }

    pub fn was_downloaded(&self, media: &WdrMedia) -> bool {
        let new_score = media.media_resource.score();
        self.list
            .iter()
            .filter(|wdrmedia| wdrmedia.tracker_data == media.tracker_data)
            .any(|wdrmedia| wdrmedia.media_resource.score() >= new_score)
    }

    pub fn mark_downloaded(media: WdrMedia) {
        let mut list = Self::new().list;
        list.push(media);
        list.sort();
        let content = serde_yaml::to_string(&list).unwrap();
        std::fs::write(DOWNLOADED_PATH, content).expect("failed to write downloaded.yaml");
    }
}

#[allow(clippy::min_ident_chars)]
#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use url::Url;

    use crate::wdr_media::{Captions, MediaFormat, MediaResource, MediaResources, TrackerData};

    use super::*;

    static A0: Lazy<WdrMedia> = Lazy::new(|| WdrMedia {
        tracker_data: TrackerData {
            id: "a".into(),
            air_time: "42".into(),
            title: "42".into(),
        },
        media_resource: MediaResources {
            preview_image: None,
            dflt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            alt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            captions_hash: Captions { srt: None },
        },
    });
    static A1: Lazy<WdrMedia> = Lazy::new(|| WdrMedia {
        tracker_data: TrackerData {
            id: "a".into(),
            air_time: "42".into(),
            title: "42".into(),
        },
        media_resource: MediaResources {
            preview_image: None,
            dflt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: Url::parse("https://edjopato.de").ok(),
                ad_video: None,
            },
            alt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            captions_hash: Captions { srt: None },
        },
    });
    static A2: Lazy<WdrMedia> = Lazy::new(|| WdrMedia {
        tracker_data: TrackerData {
            id: "a".into(),
            air_time: "42".into(),
            title: "42".into(),
        },
        media_resource: MediaResources {
            preview_image: None,
            dflt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: Url::parse("https://edjopato.de").ok(),
                ad_video: None,
            },
            alt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            captions_hash: Captions {
                srt: Url::parse("https://edjopato.de").ok(),
            },
        },
    });
    static B: Lazy<WdrMedia> = Lazy::new(|| WdrMedia {
        tracker_data: TrackerData {
            id: "b".into(),
            air_time: "42".into(),
            title: "42".into(),
        },
        media_resource: MediaResources {
            preview_image: None,
            dflt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            alt: MediaResource {
                media_format: MediaFormat::Mp4,
                video: Url::parse("https://edjopato.de").unwrap(),
                sl_video: None,
                ad_video: None,
            },
            captions_hash: Captions { srt: None },
        },
    });

    #[test]
    fn score() {
        assert_eq!(0, A0.media_resource.score());
        assert_eq!(1, A1.media_resource.score());
        assert_eq!(2, A2.media_resource.score());
        assert_eq!(0, B.media_resource.score());
    }

    #[test]
    fn empty_wasnt_downloaded() {
        let downloaded = Downloaded { list: vec![] };
        assert!(!downloaded.was_downloaded(&A0));
        assert!(!downloaded.was_downloaded(&A1));
        assert!(!downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn a_differs_b() {
        let downloaded = Downloaded {
            list: vec![A0.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(!downloaded.was_downloaded(&B));

        let downloaded = Downloaded {
            list: vec![B.clone()],
        };
        assert!(!downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a0() {
        let downloaded = Downloaded {
            list: vec![A0.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(!downloaded.was_downloaded(&A1));
        assert!(!downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a1() {
        let downloaded = Downloaded {
            list: vec![A1.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&A1));
        assert!(!downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a2() {
        let downloaded = Downloaded {
            list: vec![A2.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&A1));
        assert!(downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a0_and_a1() {
        let downloaded = Downloaded {
            list: vec![A0.clone(), A1.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&A1));
        assert!(!downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a0_and_a2() {
        let downloaded = Downloaded {
            list: vec![A0.clone(), A2.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&A1));
        assert!(downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }

    #[test]
    fn upgrade_a1_and_a2() {
        let downloaded = Downloaded {
            list: vec![A1.clone(), A2.clone()],
        };
        assert!(downloaded.was_downloaded(&A0));
        assert!(downloaded.was_downloaded(&A1));
        assert!(downloaded.was_downloaded(&A2));
        assert!(!downloaded.was_downloaded(&B));
    }
}
