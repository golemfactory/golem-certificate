use super::*;

use golem_certificate::{Key, SignedCertificate};
use serde_json::Value;

pub struct SignatureEditor {
    allow_self_sign: bool,
    data: Value,
    signing_key: Option<Key>,
    signing_certificate: Option<SignedCertificate>,
}

