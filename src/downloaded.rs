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
            .filter(|o| o.tracker_data == media.tracker_data)
            .any(|o| o.media_resource.score() >= new_score)
    }

    pub fn mark_downloaded(media: WdrMedia) {
        let mut list = Self::new().list;
        list.push(media);
        list.sort();
        let content = serde_yaml::to_string(&list).unwrap();
        std::fs::write(DOWNLOADED_PATH, content).expect("failed to write downloaded.yaml");
    }
}
