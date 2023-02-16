use std::path::Path;
use std::process::{Command, Stdio};

use lazy_regex::{lazy_regex, Lazy, Regex};
use tempfile::NamedTempFile;
use url::Url;

use crate::temporary::get_tempfile;

fn ffmpeg() -> Command {
    let mut command = Command::new("nice");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(["-n", "15"])
        .arg("ffmpeg")
        .arg("-y")
        .args(["-v", "error"]);
    command
}

fn run_command(mut command: Command) -> anyhow::Result<()> {
    if !command.status()?.success() {
        let command_line = command
            .get_args()
            .skip(2)
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
    pub height: u16,
    pub width: u16,
    /// In seconds
    pub duration: u32,
}

impl VideoStats {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        static DURATION: Lazy<Regex> = lazy_regex!(r"Duration: (\d{2}):(\d{2}):(\d{2})\.");
        static RESOLUTION: Lazy<Regex> = lazy_regex!(r", (\d+)x(\d+) \[");

        let output = Command::new("ffprobe")
            .arg("-hide_banner")
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
