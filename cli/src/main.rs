use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser};
use hex::ToHex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use golem_certificate as gcert;
#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod ui;

#[derive(Parser)]
enum GolemCertificateCli {
    CreateKeyPair { key_pair_path: PathBuf },
    Fingerprint { input_file_path: PathBuf },
    SelfSignCertificate(SelfSignArguments),
    Sign(SignArguments),
    Verify { signed_file_path: PathBuf },
    #[cfg(feature = "tui")]
    Ui,
}

#[derive(Args)]
struct SelfSignArguments {
    certificate_path: PathBuf,
    private_key_path: PathBuf,
}

#[derive(Args)]
struct SignArguments {
    input_file_path: PathBuf,
    certificate_path: PathBuf,
    private_key_path: PathBuf,
}

enum FileType {
    Certificate,
    NodeDescriptor,
}

impl FileType {
    fn signed_property(&self) -> String {
        match self {
            FileType::Certificate => "certificate",
            FileType::NodeDescriptor => "nodeDescriptor",
        }
        .to_string()
    }
}

fn determine_file_type(json_data: &Value) -> Result<FileType> {
    json_data["$schema"]
        .as_str()
        .map(|schema| match schema {
            "https://golem.network/schemas/v1/certificate.schema.json" => Ok(FileType::Certificate),
            "https://golem.network/schemas/v1/node-descriptor.schema.json" => {
                Ok(FileType::NodeDescriptor)
            }
            _ => Err(anyhow!("Unknown json structure {schema}")),
        })
        .unwrap_or_else(|| Err(anyhow!("Unknown json structure, missing $schema property")))
}

fn save_json_to_file<C: ?Sized + Serialize>(path: impl AsRef<Path>, content: &C) -> Result<()> {
    let mut writer = BufWriter::new(fs::File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, content)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn save_json_with_extension<C: ?Sized + Serialize>(
    path: &Path,
    content: &C,
    extension: &str,
) -> Result<()> {
    let mut modified_path = path.to_path_buf();
    modified_path.set_extension(extension);
    save_json_to_file(modified_path, content)
}

fn create_key_pair(key_pair_path: &Path) -> Result<()> {
    let key_pair = gcert::create_key_pair();
    save_json_with_extension(key_pair_path, &key_pair.public_key, "pub")?;
    save_json_with_extension(key_pair_path, &key_pair.private_key, "key")
}

fn print_fingerprint(input_file_path: &PathBuf) -> Result<()> {
    let input_json = deserialize_from_file::<Value>(input_file_path)?;
    let signed_property = determine_file_type(&input_json)?.signed_property();
    let signed_data = &input_json[signed_property];
    let fingerprint = gcert::create_default_hash(signed_data)?;
    println!("{}", fingerprint.encode_hex::<String>());
    Ok(())
}

fn save_signed_json<C: ?Sized + Serialize>(path: &Path, content: &C) -> Result<()> {
    save_json_with_extension(path, content, "signed.json")
}

fn deserialize_from_file<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<T> {
    let json_string = fs::read_to_string(path)?;
    serde_json::from_str(&json_string).map_err(Into::into)
}

fn sign_json_value(
    value: &Value,
    private_key_path: &PathBuf,
) -> Result<(gcert::SignatureAlgorithm, Vec<u8>)> {
    let private_key = deserialize_from_file(private_key_path)?;
    gcert::sign_json(value, &private_key)
}

fn add_signature<S: Serialize>(value: &mut Value, signature: gcert::Signature<S>) -> Result<()> {
    value["signature"] = serde_json::to_value(signature)?;
    Ok(())
}

fn self_sign_certificate(self_sign_arguments: &SelfSignArguments) -> Result<()> {
    let mut certificate = deserialize_from_file::<Value>(&self_sign_arguments.certificate_path)?;
    let file_type = determine_file_type(&certificate)?;
    let signed_property = match file_type {
        FileType::Certificate => Ok(file_type.signed_property()),
        _ => Err(anyhow!(
            "Provided path does not point to a Golem Certificate {:?}",
            self_sign_arguments.certificate_path
        )),
    }?;
    let signed_data = &certificate[signed_property];
    let (algorithm, signature_value) =
        sign_json_value(signed_data, &self_sign_arguments.private_key_path)?;
    let signature = gcert::Signature::create_self_signed(algorithm, signature_value);
    add_signature(&mut certificate, signature)?;
    save_signed_json(&self_sign_arguments.certificate_path, &certificate)
}

fn sign_json(sign_arguments: &SignArguments) -> Result<()> {
    let mut input_json = deserialize_from_file::<Value>(&sign_arguments.input_file_path)?;
    let signed_property = determine_file_type(&input_json)?.signed_property();
    let signed_data = &input_json[signed_property];
    let (algorithm, signature_value) =
        sign_json_value(signed_data, &sign_arguments.private_key_path)?;
    let certificate = deserialize_from_file(&sign_arguments.certificate_path)?;
    let signature = gcert::Signature::create(algorithm, signature_value, certificate);
    add_signature(&mut input_json, signature)?;
    save_signed_json(&sign_arguments.input_file_path, &input_json)
}

/// Determines type of signed file (Certificate or Node Descriptor) and then verifies its signature.
/// # Arguments
/// * `signed_file` path to signed file
/// * `timestamp` optional timestamp to verify validity
fn verify_signature(signed_file: &PathBuf, timestamp: Option<DateTime<Utc>>) -> Result<()> {
    let signed_json = deserialize_from_file::<Value>(signed_file)?;
    match determine_file_type(&signed_json)? {
        FileType::Certificate => gcert::validate_certificate(signed_json, timestamp)
            .map(|result| println!("{:?}", result))
            .map_err(Into::into),
        FileType::NodeDescriptor => gcert::validate_node_descriptor(signed_json, timestamp)
            .map(|result| println!("{:?}", result))
            .map_err(Into::into),
    }
}

fn main() -> Result<()> {
    match GolemCertificateCli::parse() {
        GolemCertificateCli::CreateKeyPair { key_pair_path } => create_key_pair(&key_pair_path),
        GolemCertificateCli::Fingerprint { input_file_path } => print_fingerprint(&input_file_path),
        GolemCertificateCli::SelfSignCertificate(self_sign_arguments) => {
            self_sign_certificate(&self_sign_arguments)
        }
        GolemCertificateCli::Sign(sign_arguments) => sign_json(&sign_arguments),
        GolemCertificateCli::Verify { signed_file_path } => {
            verify_signature(&signed_file_path, Some(Utc::now()))
        }
        #[cfg(feature = "tui")]
        GolemCertificateCli::Ui => app::start(),
    }
}
