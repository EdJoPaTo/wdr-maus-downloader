use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate, Timelike, Weekday};
use serde::{Deserialize, Serialize};

const DAILY_PATH: &str = "daily.yaml";

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Job {
    AktuelleSunday,
    AktuelleCheckup,
    SachgeschichteMorning,
    SachgeschichteEvening,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Daily {
    day: NaiveDate,
    jobs: HashMap<Job, bool>,
}

impl Default for Daily {
    fn default() -> Self {
        Self {
            day: Local::today().naive_local(),
            jobs: HashMap::new(),
        }
    }
}

impl Daily {
    pub fn new() -> Self {
        let file: Self = std::fs::read_to_string(DAILY_PATH)
            .map(|content| serde_yaml::from_str(&content).expect("daily.yaml format error"))
            .unwrap_or_default();

        let today = Local::today().naive_local();
        if file.day == today {
            file
        } else {
            Self::default()
        }
    }

    fn write(&self) {
        let content = serde_yaml::to_string(self).unwrap();
        std::fs::write(DAILY_PATH, content).expect("failed to write daily.yaml");
    }

    pub fn mark_successful(&mut self, job: Job) {
        self.jobs.insert(job, true);
        self.write();
    }

    fn is_done(&self, job: Job) -> bool {
        self.jobs.get(&job).map_or(false, |o| *o)
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
