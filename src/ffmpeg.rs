use std::process::{Command, Stdio};

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
