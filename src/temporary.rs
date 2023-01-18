use tempfile::NamedTempFile;

pub fn get_tempfile(suffix: &'static str) -> std::io::Result<NamedTempFile> {
    tempfile::Builder::new()
        .prefix("wdr-maus-")
        .suffix(suffix)
        .tempfile()
}
