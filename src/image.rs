use std::io::BufWriter;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::NamedTempFile;
use url::Url;

use crate::temporary::get_tempfile;

pub fn download_jpg(url: &Url) -> anyhow::Result<NamedTempFile> {
    let mut reader = ureq::get(url.as_str()).call()?.into_body().into_reader();
    let file = get_tempfile(".jpg")?;
    {
        let mut writer = BufWriter::new(file.as_file());
        std::io::copy(&mut reader, &mut writer)?;
    }
    Ok(file)
}

pub fn resize_to_tg_thumbnail(image: &Path) -> anyhow::Result<NamedTempFile> {
    // TODO: maybe use image and/or imageproc crate

    let output = get_tempfile(".jpg")?;

    let mut command = Command::new("nice");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("magick")
        .arg(image.as_os_str())
        .args(["-sampling-factor", "4:2:0"])
        .args(["-resize", "320x320>"])
        .arg(output.path().as_os_str());

    let status = command.status().expect("failed to execute ImageMagick");
    if !status.success() {
        let command_line = command
            .get_args()
            .map(std::ffi::OsStr::to_string_lossy)
            .collect::<Vec<_>>()
            .join(" ");
        anyhow::bail!("ImageMagick exited unsuccessfully. Commandline: {command_line}");
    }

    Ok(output)
}
