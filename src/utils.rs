use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn write_file(path: impl AsRef<Path>, content: &[u8], append: bool) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)?;

    file.write_all(content)
}
