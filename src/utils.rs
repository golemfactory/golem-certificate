use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

#[allow(dead_code)]
pub fn save_json_to_file<C: ?Sized + Serialize>(path: impl AsRef<Path>, content: &C) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, content)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}
