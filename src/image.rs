use std::io::BufWriter;
use std::process::{Command, Stdio};

use tempfile::NamedTempFile;

use crate::temporary::get_tempfile;

pub fn get_thumbnail(url: &str) -> anyhow::Result<NamedTempFile> {
    // TODO: maybe use image and/or imageproc crate

    let input = {
        let mut reader = ureq::get(url).call()?.into_reader();
        let file = get_tempfile(".jpg")?;
        {
            let mut writer = BufWriter::new(file.as_file());
            std::io::copy(&mut reader, &mut writer)?;
        }
        file
    };

    let output = get_tempfile(".jpg")?;

    let mut command = Command::new("nice");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("convert")
        .arg(input.path().as_os_str())
        .args(["-sampling-factor", "4:2:0"])
        .args(["-resize", "320x320>"])
        .arg(output.path().as_os_str());

    if !command.status()?.success() {
        let command_line = command
            .get_args()
            .skip(2)
            .map(std::ffi::OsStr::to_string_lossy)
            .collect::<Vec<_>>()
            .join(" ");
        anyhow::bail!("ImageMagick exited unsuccessfully. Commandline: {command_line}");
    }

    Ok(output)
}
