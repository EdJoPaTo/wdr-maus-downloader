use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate, Timelike, Weekday};
use serde::{Deserialize, Serialize};

use crate::wdr_media::WdrMedia;

const DAILY_PATH: &str = "daily.yaml";

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Job {
    AktuelleSunday,
    AktuelleCheckup,
    SachgeschichteMorning,
    SachgeschichteEvening,
}

#[derive(Serialize, Deserialize)]
pub struct Daily {
    day: NaiveDate,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    jobs: HashMap<Job, bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    errors: Vec<WdrMedia>,
}

impl Daily {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        std::fs::read_to_string(DAILY_PATH)
            .map(|content| serde_yaml::from_str::<Self>(&content).expect("daily.yaml format error"))
            .ok()
            .filter(|file| file.day == today)
            .unwrap_or_else(|| Self {
                day: today,
                jobs: HashMap::new(),
                errors: Vec::new(),
            })
    }

    fn write(&self) {
        let content = serde_yaml::to_string(self).unwrap();
        std::fs::write(DAILY_PATH, content).expect("failed to write daily.yaml");
    }

    pub fn mark_successful(&mut self, job: Job) {
        self.jobs.insert(job, true);
        self.errors.clear();
        self.write();
    }

    pub fn mark_error(&mut self, error: WdrMedia) {
        self.errors.push(error);
        self.write();
    }

    fn is_done(&self, job: Job) -> bool {
        self.jobs.get(&job).copied().unwrap_or(false)
    }

    pub fn is_error(&self, media: &WdrMedia) -> bool {
        self.errors.contains(media)
    }

    pub fn get_next(&self) -> Option<Job> {
        let now = Local::now();
        println!(
            "check do_next… {:>2}:{:>02} {}",
            now.hour(),
            now.minute(),
            now.weekday()
        );

        if now.weekday() == Weekday::Sun && now.hour() >= 8 && now.hour() < 13 {
            Some(Job::AktuelleSunday)
        } else if !self.is_done(Job::AktuelleCheckup) && now.hour() >= 19 {
            Some(Job::AktuelleCheckup)
        } else if !self.is_done(Job::SachgeschichteMorning) && now.hour() >= 5 {
            Some(Job::SachgeschichteMorning)
        } else if !self.is_done(Job::SachgeschichteEvening) && now.hour() >= 16 {
            Some(Job::SachgeschichteEvening)
        } else {
            None
        }
    }
}
