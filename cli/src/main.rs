use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser};
use gcert::schemas::{SIGNED_CERTIFICATE_SCHEMA_ID, SIGNED_NODE_DESCRIPTOR_SCHEMA_ID};
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
    #[command(about = "Creates a new key pair")]
    CreateKeyPair {
        #[arg(
            help = "Path to save the keypair to. Public key is saved with extension set to .pub.json, signing key is saved with extension .key.json"
        )]
        key_pair_path: PathBuf,
    },
    #[command(about = "Prints fingerprint of the signed property of the input file")]
    Fingerprint {
        #[arg(help = "Path to a certificate or node descriptor")]
        input_file_path: PathBuf,
    },
    #[command(about = "Creates self-signed certificate")]
    SelfSignCertificate(SelfSignArguments),
    #[command(about = "Signs a certificate or node descriptor")]
    Sign(SignArguments),
    #[command(
        about = "Verifies the signature and other constraints of the input certificate or node descriptor"
    )]
    Verify {
        #[arg(help = "Path to a signed certificate or node descriptor")]
        signed_file_path: PathBuf,
        #[arg(value_parser = parse_timestamp)]
        #[arg(
            help = "Optional RFC 3339 formatted timestamp (ex: 2020-01-01T13:42:33Z) to verify validity. 'now' can be used to refer to current time."
        )]
        timestamp: Option<DateTime<Utc>>,
    },
    #[cfg(feature = "tui")]
    #[command(about = "Starts Golem Certificate Manager")]
    Ui,
}

#[derive(Args)]
struct SelfSignArguments {
    #[arg(
        help = "Path to the certificate to be self-signed. Signed certificate is the same path with extension set to .signed.json"
    )]
    certificate_path: PathBuf,
    #[arg(help = "Path to the signing key associated with the public key in the certificate.")]
    signing_key_path: PathBuf,
}

#[derive(Args)]
struct SignArguments {
    #[arg(
        help = "Path to the certificate or node descriptor to be signed. Signed document is saved to the same path with extension set to .signed.json"
    )]
    input_file_path: PathBuf,
    #[arg(help = "Path to the signing certificate")]
    certificate_path: PathBuf,
    #[arg(
        help = "Path to the signing key associated with the public key in the signing certificate."
    )]
    signing_key_path: PathBuf,
}

fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>> {
    if timestamp == "now" {
        Ok(Utc::now())
    } else {
        timestamp.parse::<DateTime<Utc>>().map_err(Into::into)
    }
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
            SIGNED_CERTIFICATE_SCHEMA_ID => Ok(FileType::Certificate),
            SIGNED_NODE_DESCRIPTOR_SCHEMA_ID => Ok(FileType::NodeDescriptor),
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
    signing_key_path: &PathBuf,
) -> Result<(gcert::SignatureAlgorithm, Vec<u8>)> {
    let signing_key = deserialize_from_file(signing_key_path)?;
    gcert::sign_json(value, &signing_key)
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
        sign_json_value(signed_data, &self_sign_arguments.signing_key_path)?;
    let signature = gcert::Signature::create_self_signed(algorithm, signature_value);
    add_signature(&mut certificate, signature)?;
    save_signed_json(&self_sign_arguments.certificate_path, &certificate)
}

fn sign_json(sign_arguments: &SignArguments) -> Result<()> {
    let mut input_json = deserialize_from_file::<Value>(&sign_arguments.input_file_path)?;
    let signed_property = determine_file_type(&input_json)?.signed_property();
    let signed_data = &input_json[signed_property];
    let (algorithm, signature_value) =
        sign_json_value(signed_data, &sign_arguments.signing_key_path)?;
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
        GolemCertificateCli::Verify {
            signed_file_path,
            timestamp,
        } => verify_signature(&signed_file_path, timestamp),
        #[cfg(feature = "tui")]
        GolemCertificateCli::Ui => app::start(),
    }
}
