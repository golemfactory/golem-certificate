use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Subcommand;
use golem_certificate::Key;
use openpgp_card::{CardBackend, Error, CardTransaction};
use openpgp_card_pcsc::PcscBackend;

use crate::utils::save_json_to_file;

// Details of the commands are from
// 'Functional Specification of the OpenPGP applicationon ISO Smart Card Operating Systems'
// https://gnupg.org/ftp/specs/OpenPGP-smart-card-application-3.4.1.pdf

#[derive(Subcommand)]
pub enum SmartcardCommand {
    #[command(about = "Lists all visible smartcards. If your card does not show up, try to unplug and replug it")]
    List,
    #[command(about = "Exports the public key of the OpenPGP sign key")]
    ExportPrivateKey {
        #[arg(help = "The card identifier as printed by the list command")]
        ident: String,
        #[arg(help = "Path to save the public keyr to")]
        public_key_path: PathBuf,
    }
}

pub fn smartcard(cmd: SmartcardCommand) -> Result<()> {
    use SmartcardCommand::*;
    match cmd {
        List => list(),
        ExportPrivateKey { ident, public_key_path } => export_private_key(ident, public_key_path),
    }
}

fn list() -> Result<()> {
    let cards = PcscBackend::cards(None)?;
    println!("Found: {} cards", cards.len());
    cards.into_iter().map(|mut backend| {
        let mut transaction = backend.transaction()?;
        println!("Card details:");
        let app_data = transaction.application_related_data()?;
        let app_id = app_data.application_id()?;
        println!(" Manufacturer {}", app_id.manufacturer_name());
        println!(" Ident: {}", app_id.ident());
        let fingerprints = app_data.fingerprints()?;
        let sign_fingerprint =
            fingerprints.signature().map(|f| f.to_spaced_hex()).unwrap_or("none".into());
        println!(" OpenPGP signature key fingerprint: {}", sign_fingerprint);
        Ok::<(), Error>(())
    }).collect::<Result<Vec<_>, Error>>()?;
    Ok(())
}

type Transaction<'a> = Box<dyn CardTransaction + Sync + Send + 'a>;

fn export_private_key(ident: String, public_key_path: PathBuf) -> Result<()> {
    let mut card = PcscBackend::open_by_ident(&ident, None)?;
    let mut transaction = card.transaction()?;
    let public_key = read_public_key(&mut transaction)?;
    save_json_to_file(public_key_path, &public_key)?;
    Ok(())
}

fn verify_key_algo<'a>(transaction: &mut Transaction<'a>) -> Result<()> {
    let app_data = transaction.application_related_data()?;
    let key_algorithm = app_data.algorithm_attributes(openpgp_card::KeyType::Signing)?;
    match key_algorithm {
        openpgp_card::algorithm::Algo::Rsa(_) => Err(anyhow!("RSA signing keys are not supported")),
        openpgp_card::algorithm::Algo::Ecc(attr) => {
            if attr.ecc_type() != openpgp_card::crypto_data::EccType::EdDSA {
                Err(anyhow!("Only EdDSA signing keys are supported. Not supported: {:?}", attr.ecc_type()))
            } else if attr.curve() != openpgp_card::algorithm::Curve::Ed25519 {
                Err(anyhow!("Only ed25519 curve is supported. Not supported: {:?}", attr.ecc_type()))
            } else {
                Ok(())
            }
        },
        _ => Err(anyhow!("Unknown signing key algorithm")),
    }.map_err(Into::into)
}

fn read_public_key<'a>(transaction: &mut Transaction<'a>) -> Result<Key> {
    verify_key_algo(transaction)?;
    // Specification section
    // 7.2.14 GENERATE ASYMMETRIC KEY PAIR
    let cmd_result = transaction.transmit(&[0x00, 0x47, 0x81, 0x00, 0x05, 0xb6, 0x03, 0x84, 0x01, 0x01, 0x00], 40)?;
    if cmd_result.len() != 39 {
        Err(anyhow!("Unexpected response length. Response {:x?}", cmd_result))
    } else {
        let key_bytes: Vec<u8> = cmd_result[5..37].into();
        Ok(key_bytes.try_into().unwrap())
    }
}
