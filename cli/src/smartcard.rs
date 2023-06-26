use clap::Subcommand;
use openpgp_card::{CardBackend, Error};
use openpgp_card_pcsc::PcscBackend;

type ScResult = Result<(), Error>;

#[derive(Subcommand)]
pub enum SmartcardCommand {
    #[command(about = "Lists all visible smartcards. If your card does not show up, try to unplug and replug it")]
    List,
}

pub fn smartcard(cmd: SmartcardCommand) -> ScResult {
    match cmd {
        SmartcardCommand::List => list(),
    }
}

fn list() -> ScResult {
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