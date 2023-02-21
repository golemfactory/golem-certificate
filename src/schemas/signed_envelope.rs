use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

//TODO additionalproperties=false
//TODO Rafał vec to set
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedEnvelope {
    //TODO add $schema
    //TODO add $schema inside signed_data
    pub signed_data: Box<RawValue>,
    //TODO Rafał Change to vec
    pub signatures: Box<Signature>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    //TODO add algorithm & signature
    pub signer: Signer,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Signer {
    //TODO Rafał rename to self in serde + untagged
    //TODO maybe add Other?
    SelfSigned,
    Certificate(SignedEnvelope),
}
