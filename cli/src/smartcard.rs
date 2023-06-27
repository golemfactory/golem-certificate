use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Subcommand;
use golem_certificate::{
    create_default_hash, schemas::SIGNED_CERTIFICATE_SCHEMA_ID, EncryptionAlgorithm, Key,
    Signature, SignatureAlgorithm, SignedCertificate,
};
use openpgp_card::{CardBackend, CardTransaction, Error};
use openpgp_card_pcsc::PcscBackend;
use serde_json::Value;

use crate::{
    add_signature,
    utils::{deserialize_from_file, determine_file_type, save_json_to_file},
};

// Details of the commands are from
// 'Functional Specification of the OpenPGP applicationon ISO Smart Card Operating Systems'
// https://gnupg.org/ftp/specs/OpenPGP-smart-card-application-3.4.1.pdf

#[derive(Subcommand)]
pub enum SmartcardCommand {
    #[command(
        about = "Lists all visible smartcards. If your card does not show up, try to unplug and replug it"
    )]
    List,
    #[command(about = "Exports the public key of the OpenPGP sign key")]
    ExportPublicKey {
        #[arg(help = "The card identifier as printed by the list command")]
        ident: String,
        #[arg(help = "Path to save the public key to")]
        public_key_path: PathBuf,
    },
    #[command(about = "Signs a certificate or node descriptor")]
    Sign {
        #[arg(help = "The card identifier as printed by the list command")]
        ident: String,
        #[arg(
            help = "Path to the certificate or node descriptor to be signed. Signed document is saved to the same path with extension set to .signed.json"
        )]
        input_file_path: PathBuf,
        #[arg(help = "Path to the signing certificate")]
        certificate_path: PathBuf,
    },
    #[command(
        about = "Create self-signed certificate, replacing the public key with the one from the smartcard"
    )]
    SelfSignCertificate {
        #[arg(help = "The card identifier as printed by the list command")]
        ident: String,
        #[arg(
            help = "Path to the certificate to be self-signed. Signed certificate is saved to the same path with extension set to .signed.json"
        )]
        certificate_path: PathBuf,
    },
}

pub fn smartcard(cmd: SmartcardCommand) -> Result<()> {
    use SmartcardCommand::*;
    match cmd {
        List => list(),
        ExportPublicKey {
            ident,
            public_key_path,
        } => export_public_key(ident, public_key_path),
        Sign {
            ident,
            input_file_path,
            certificate_path,
        } => sign_json_document(ident, input_file_path, certificate_path),
        SelfSignCertificate {
            certificate_path,
            ident,
        } => self_sign_certificate(ident, certificate_path),
    }
}

fn list() -> Result<()> {
    let cards = PcscBackend::cards(None)?;
    println!("Found: {} cards", cards.len());
    cards
        .into_iter()
        .map(|mut backend| {
            let mut transaction = backend.transaction()?;
            println!("Card details:");
            let app_data = transaction.application_related_data()?;
            let app_id = app_data.application_id()?;
            println!(" Manufacturer {}", app_id.manufacturer_name());
            println!(" Ident: {}", app_id.ident());
            let fingerprints = app_data.fingerprints()?;
            let sign_fingerprint = fingerprints
                .signature()
                .map(|f| f.to_spaced_hex())
                .unwrap_or("none".into());
            println!(" OpenPGP signature key fingerprint: {}", sign_fingerprint);
            Ok::<(), Error>(())
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(())
}

fn export_public_key(ident: String, public_key_path: PathBuf) -> Result<()> {
    let mut card = open_card(&ident)?;
    let mut transaction = card.transaction()?;
    verify_key_algo(&mut transaction)?;
    let public_key = read_public_key(&mut transaction)?;
    save_json_to_file(public_key_path, &public_key)?;
    Ok(())
}

fn sign_json_document(
    ident: String,
    document_path: PathBuf,
    certificate_path: PathBuf,
) -> Result<()> {
    let mut document = deserialize_from_file::<Value>(&document_path)?;
    let signed_property = determine_file_type(&document)?.signed_property();
    let signed_data = &document[signed_property];
    let mut card = open_card(&ident)?;
    let mut transaction = card.transaction()?;
    let public_key = read_public_key(&mut transaction)?;
    let certificate: SignedCertificate = deserialize_from_file(&certificate_path)?;
    if serde_json::to_value(public_key)? != certificate.certificate["publicKey"] {
        Err(anyhow!(
            "Public key in the signign certificate does not match the public key from the card"
        ))
    } else {
        let (algorithm, signature_value) = sign_json(&mut transaction, signed_data)?;
        let signature = Signature::create(algorithm, signature_value, certificate);
        add_signature(&mut document, signature)?;
        save_json_to_file(document_path.with_extension("signed.json"), &document)
    }
}

fn self_sign_certificate(ident: String, certificate_path: PathBuf) -> Result<()> {
    let mut certificate = deserialize_from_file::<Value>(&certificate_path)
        .map_err(|e| anyhow!("Failed to read certificate: {}", e))?;
    let mut card = open_card(&ident)?;
    let mut transaction = card.transaction()?;
    let public_key = read_public_key(&mut transaction)?;
    certificate["certificate"]["publicKey"] = serde_json::to_value(public_key)?;
    certificate["$schema"] = SIGNED_CERTIFICATE_SCHEMA_ID.into();
    let (signature_algorithm, signature_bytes) =
        sign_json(&mut transaction, &certificate["certificate"])?;
    let signature = Signature::create_self_signed(signature_algorithm, signature_bytes);
    add_signature(&mut certificate, signature)?;
    save_json_to_file(certificate_path.with_extension("signed.json"), &certificate)?;
    Ok(())
}

type Transaction<'a> = Box<dyn CardTransaction + Sync + Send + 'a>;

fn sign_json(
    transaction: &mut Transaction<'_>,
    signed_data: &Value,
) -> Result<(SignatureAlgorithm, Vec<u8>)> {
    let hash = create_default_hash(signed_data)?;
    verify_key_algo(transaction)?;
    login(transaction)?;
    let signature_bytes = sign_hash(transaction, &hash)?;
    let signature_algorithm = SignatureAlgorithm {
        encryption: EncryptionAlgorithm::EdDSAOpenPGP,
        ..Default::default()
    };
    Ok((signature_algorithm, signature_bytes))
}

fn open_card(ident: &str) -> Result<PcscBackend, Error> {
    PcscBackend::open_by_ident(ident, None)
}

fn verify_key_algo(transaction: &mut Transaction<'_>) -> Result<()> {
    let app_data = transaction.application_related_data()?;
    let key_algorithm = app_data.algorithm_attributes(openpgp_card::KeyType::Signing)?;
    match key_algorithm {
        openpgp_card::algorithm::Algo::Rsa(_) => Err(anyhow!("RSA signing keys are not supported")),
        openpgp_card::algorithm::Algo::Ecc(attr) => {
            if attr.ecc_type() != openpgp_card::crypto_data::EccType::EdDSA {
                Err(anyhow!(
                    "Only EdDSA signing keys are supported. Found: {:?}",
                    attr.ecc_type()
                ))
            } else if attr.curve() != openpgp_card::algorithm::Curve::Ed25519 {
                Err(anyhow!(
                    "Only ed25519 curve is supported. Found: {:?}",
                    attr.ecc_type()
                ))
            } else {
                Ok(())
            }
        }
        _ => Err(anyhow!("Unknown signing key algorithm")),
    }
    .map_err(Into::into)
}

fn read_public_key(transaction: &mut Transaction<'_>) -> Result<Key> {
    // Specification section
    // 7.2.14 GENERATE ASYMMETRIC KEY PAIR
    const CMD: [u8; 11] = [
        0x00, 0x47, 0x81, 0x00, 0x05, 0xb6, 0x03, 0x84, 0x01, 0x01, 0x00,
    ];
    let cmd_result = transaction.transmit(&CMD, 40)?;
    if cmd_result.len() != 39 {
        Err(anyhow!(
            "Unexpected response length. Response {:x?}",
            cmd_result
        ))
    } else {
        let key_bytes: Vec<u8> = cmd_result[5..37].into();
        Ok(key_bytes.try_into().unwrap())
    }
}

fn login(transaction: &mut Transaction<'_>) -> Result<()> {
    let pin = rpassword::prompt_password("Enter PIN: ")?;
    let mut cmd = vec![0x00, 0x20, 0x00, 0x81];
    cmd.push(pin.len() as u8);
    cmd.extend_from_slice(pin.as_bytes());
    let cmd_result = transaction.transmit(&cmd, 2)?;
    if cmd_result != vec![0x90, 0x00] {
        Err(anyhow!("Login failed. Response: {:x?}", cmd_result))
    } else {
        Ok(())
    }
}

fn sign_hash(transaction: &mut Transaction<'_>, hash: &[u8]) -> Result<Vec<u8>> {
    let mut cmd = vec![0x00, 0x2a, 0x9e, 0x9a];
    cmd.push(hash.len() as u8);
    cmd.extend_from_slice(hash);
    cmd.push(0x00);
    let cmd_result = transaction.transmit(&cmd, 1024)?;
    if cmd_result.len() != 66 {
        Err(anyhow!(
            "Unexpected response length. Response {:x?}",
            cmd_result
        ))
    } else if cmd_result[64..66] != [0x90, 0x00] {
        Err(anyhow!("Signing failed. Response: {:x?}", cmd_result))
    } else {
        let signature = &cmd_result[..cmd_result.len() - 2];
        Ok(signature.into())
    }
}
