use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Context as _;
use lazy_regex::regex;
use tempfile::NamedTempFile;
use url::Url;

use crate::temporary::get_tempfile;

fn ffmpeg() -> Command {
    let mut command = Command::new("nice");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-n17")
        .arg("ffmpeg")
        .arg("-y")
        .args(["-v", "error"]);
    command
}

fn run_command(mut command: Command) -> anyhow::Result<()> {
    let status = command.status().expect("failed to execute ffmpeg");
    if !status.success() {
        let command_line = command
            .get_args()
            .skip(1)
            .map(std::ffi::OsStr::to_string_lossy)
            .collect::<Vec<_>>()
            .join(" ");
        anyhow::bail!("ffmpeg exited unsuccessfully. Commandline: {command_line}");
    }
    Ok(())
}

pub fn download(video: &Url, caption_srt: Option<&Url>) -> anyhow::Result<NamedTempFile> {
    let file = get_tempfile(".mp4")?;
    let mut command = ffmpeg();

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
        .args(["-c:v", "libx265"])
        .arg(file.path().as_os_str());

    run_command(command)?;
    Ok(file)
}

pub fn extract_video_thumbnail(input: &Path) -> anyhow::Result<NamedTempFile> {
    let output = get_tempfile(".jpg")?;
    let mut command = ffmpeg();
    command
        .arg("-i")
        .arg(input.as_os_str())
        .args(["-vf", "thumbnail", "-frames:v", "1"])
        .arg(output.path().as_os_str());
    run_command(command)?;
    Ok(output)
}

pub struct VideoStats {
    pub height: u32,
    pub width: u32,
    /// In seconds
    pub duration: u32,
}

impl VideoStats {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let output = Command::new("ffprobe")
            .arg("-hide_banner")
            .arg(path.as_os_str())
            .output()
            .expect("failed to execute ffprobe");
        let output = String::from_utf8(output.stderr).expect("ffprobe provided non utf8 output");

        let duration = {
            let captures = regex!(r"Duration: (\d{2}):(\d{2}):(\d{2})\.")
                .captures(&output)
                .context("duration not found in ffprobe output")?;
            let hours = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
            let minutes = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let seconds = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
            (((hours * 60) + minutes) * 60) + seconds
        };

        let (width, height) = {
            let captures = regex!(r", (\d+)x(\d+) \[")
                .captures(&output)
                .context("resolution not found in ffprobe output")?;
            let width = captures.get(1).unwrap().as_str().parse().unwrap();
            let height = captures.get(2).unwrap().as_str().parse().unwrap();
            (width, height)
        };

        Ok(Self {
            height,
            width,
            duration,
        })
    }
}
