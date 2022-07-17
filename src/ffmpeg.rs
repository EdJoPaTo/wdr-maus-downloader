use std::path::Path;
use std::process::{Command, Stdio};

use once_cell::sync::Lazy;
use regex::Regex;
use tempfile::NamedTempFile;
use url::Url;

pub fn download(video: &Url, caption_srt: Option<&Url>) -> anyhow::Result<NamedTempFile> {
    let mut command = Command::new("nice");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(["-n", "15"])
        .arg("ffmpeg")
        .arg("-y")
        .args(["-v", "error"]);

    // Only short sample for faster finish
    #[cfg(debug_assertions)]
    command.args(["-t", "5"]);

    command.args(["-i", video.as_ref()]);

    if let Some(caption) = caption_srt {
        command.args(["-i", caption.as_ref()]);
    }

    command
        .args(["-c", "copy"])
        .args(["-c:s", "mov_text"])
        .args(["-c:v", "libx265"]);

    let file = get_tempfile(".mp4")?;
    command.arg(file.path().as_os_str());

    if !command.status()?.success() {
        let command_line = command
            .get_args()
            .skip(2)
            .map(std::ffi::OsStr::to_string_lossy)
            .collect::<Vec<_>>()
            .join(" ");
        anyhow::bail!(
            "ffmpeg exited unsuccessfully. Commandline: {}",
            command_line
        );
    }

    Ok(file)
}

fn get_tempfile(suffix: &str) -> std::io::Result<NamedTempFile> {
    tempfile::Builder::new()
        .prefix("wdr-maus-")
        .suffix(suffix)
        .tempfile()
}

pub struct VideoStats {
    pub height: u16,
    pub width: u16,
    /// In seconds
    pub duration: u32,
}

impl VideoStats {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        static DURATION: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.").unwrap());
        static RESOLUTION: Lazy<Regex> = Lazy::new(|| Regex::new(r", (\d+)x(\d+) \[").unwrap());

        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-i")
            .arg(path.as_os_str())
            .output()
            .expect("failed to execute ffmpeg");
        let output = String::from_utf8(output.stderr).expect("ffmpeg provided non utf8 output");

        let duration = {
            let captures = DURATION
                .captures(&output)
                .ok_or_else(|| anyhow::anyhow!("duration not found in ffmpeg output"))?;
            let hours = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
            let minutes = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let seconds = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
            (((hours * 60) + minutes) * 60) + seconds
        };

        let (width, height) = {
            let captures = RESOLUTION
                .captures(&output)
                .ok_or_else(|| anyhow::anyhow!("resolution not found in ffmpeg output"))?;
            let width = captures.get(1).unwrap().as_str().parse::<u16>().unwrap();
            let height = captures.get(2).unwrap().as_str().parse::<u16>().unwrap();
            (width, height)
        };

        Ok(Self {
            height,
            width,
            duration,
        })
    }
}
