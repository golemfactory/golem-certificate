use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use golem_certificate::schemas::{
    SIGNED_CERTIFICATE_SCHEMA_ID, SIGNED_NODE_DESCRIPTOR_SCHEMA_ID,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn save_json_to_file<C: ?Sized + Serialize>(path: impl AsRef<Path>, content: &C) -> Result<()> {
    let mut writer = BufWriter::new(fs::File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, content)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub enum FileType {
    Certificate,
    NodeDescriptor,
}

impl FileType {
    pub fn signed_property(&self) -> String {
        match self {
            FileType::Certificate => "certificate",
            FileType::NodeDescriptor => "nodeDescriptor",
        }
        .to_string()
    }
}

pub fn determine_file_type(json_data: &Value) -> Result<FileType> {
    json_data["$schema"]
        .as_str()
        .map(|schema| match schema {
            SIGNED_CERTIFICATE_SCHEMA_ID => Ok(FileType::Certificate),
            SIGNED_NODE_DESCRIPTOR_SCHEMA_ID => Ok(FileType::NodeDescriptor),
            _ => Err(anyhow!("Unknown json structure {schema}")),
        })
        .unwrap_or_else(|| Err(anyhow!("Unknown json structure, missing $schema property")))
}

pub fn save_json_with_extension<C: ?Sized + Serialize>(
    path: &Path,
    content: &C,
    extension: &str,
) -> Result<()> {
    let mut modified_path = path.to_path_buf();
    modified_path.set_extension(extension);
    save_json_to_file(modified_path, content)
}

pub fn save_signed_json<C: ?Sized + Serialize>(path: &Path, content: &C) -> Result<()> {
    save_json_with_extension(path, content, "signed.json")
}

pub fn deserialize_from_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let json_string = fs::read_to_string(path)?;
    serde_json::from_str(&json_string).map_err(Into::into)
}
