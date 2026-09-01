use anyhow::Result;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};

use ed25519_dalek::{Signature as EdDSASignature, Signer, SigningKey, VerifyingKey};
use rand::{rand_core::UnwrapErr, rngs::SysRng};

use crate::schemas::signature::SignatureAlgorithm;
use crate::serde_utils::{bytes_to_hex, hex_to_bytes};
use crate::Error;

const ED25519_SCALAR_ORDER: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HashAlgorithm {
    Sha224,
    Sha256,
    Sha384,
    #[default]
    Sha512,
    Sha3_224,
    Sha3_256,
    Sha3_384,
    Sha3_512,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub enum EncryptionAlgorithm {
    #[default]
    EdDSA,
    EdDSAOpenPGP,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Key {
    algorithm: EncryptionAlgorithm,
    #[serde(serialize_with = "bytes_to_hex", deserialize_with = "hex_to_bytes")]
    key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Value>,
}

impl From<[u8; 32]> for Key {
    fn from(value: [u8; 32]) -> Self {
        Self {
            algorithm: EncryptionAlgorithm::EdDSA,
            parameters: Some(json!({ "scheme": "Ed25519" })),
            key: value.into(),
        }
    }
}

pub struct KeyPair {
    pub public_key: Key,
    pub private_key: Key,
}

pub fn create_key_pair() -> KeyPair {
    let signing_key = SigningKey::generate(&mut UnwrapErr(SysRng));
    KeyPair {
        public_key: signing_key.verifying_key().to_bytes().into(),
        private_key: signing_key.to_bytes().into(),
    }
}

pub fn create_default_hash(value: &Value) -> Result<Vec<u8>, Error> {
    create_hash(value, &HashAlgorithm::default())
}

pub fn create_hash(value: &Value, hash_algorithm: &HashAlgorithm) -> Result<Vec<u8>, Error> {
    serde_json_canonicalizer::to_vec(value)
        .map(|canonical_json| create_digest(canonical_json, hash_algorithm))
        .map_err(|e| Error::JcsSerializationError(e.to_string()))
}

fn create_digest(input: impl AsRef<[u8]>, hash_algorithm: &HashAlgorithm) -> Vec<u8> {
    // Digest trait and the output hash contains the size so we cannot create a common variable prior to converting it into a Vec<u8>
    match hash_algorithm {
        HashAlgorithm::Sha224 => Sha224::digest(input).into_iter().collect(),
        HashAlgorithm::Sha256 => Sha256::digest(input).into_iter().collect(),
        HashAlgorithm::Sha384 => Sha384::digest(input).into_iter().collect(),
        HashAlgorithm::Sha512 => Sha512::digest(input).into_iter().collect(),
        HashAlgorithm::Sha3_224 => Sha3_224::digest(input).into_iter().collect(),
        HashAlgorithm::Sha3_256 => Sha3_256::digest(input).into_iter().collect(),
        HashAlgorithm::Sha3_384 => Sha3_384::digest(input).into_iter().collect(),
        HashAlgorithm::Sha3_512 => Sha3_512::digest(input).into_iter().collect(),
    }
}

pub fn sign_json(value: &Value, private_key: &Key) -> Result<(SignatureAlgorithm, Vec<u8>)> {
    validate_ed25519_key(private_key)?;
    let canonical_json = serde_json_canonicalizer::to_vec(value)?;
    let secret_key_bytes: &[u8; 32] = private_key
        .key
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid Ed25519 private key length"))?;
    let signing_key = SigningKey::from_bytes(secret_key_bytes);
    let signature_value = sign_bytes(canonical_json, &signing_key);
    let algorithm = SignatureAlgorithm::default();
    Ok((algorithm, signature_value))
}

fn sign_bytes(bytes: impl AsRef<[u8]>, signing_key: &SigningKey) -> Vec<u8> {
    signing_key.sign(bytes.as_ref()).to_bytes().into()
}

pub fn verify_signature_json(
    value: &Value,
    signature_algorithm: &SignatureAlgorithm,
    signature_value: impl AsRef<[u8]>,
    public_key: &Key,
) -> Result<(), Error> {
    validate_ed25519_key(public_key)?;
    if !matches!(signature_algorithm.hash, HashAlgorithm::Sha512) {
        return Err(Error::UnsupportedSignatureAlgorithm);
    }

    let canonical_json = serde_json_canonicalizer::to_vec(value)
        .map_err(|e| Error::JcsSerializationError(e.to_string()))?;
    let eddsa_signature = parse_ed25519_signature(signature_value.as_ref())?;
    let public_key_bytes: &[u8; 32] = public_key
        .key
        .as_slice()
        .try_into()
        .map_err(|_| Error::InvalidPublicKey)?;
    let public_key =
        VerifyingKey::from_bytes(public_key_bytes).map_err(|_| Error::InvalidPublicKey)?;
    match signature_algorithm.encryption {
        EncryptionAlgorithm::EdDSA => verify_bytes(canonical_json, &eddsa_signature, &public_key),
        EncryptionAlgorithm::EdDSAOpenPGP => {
            verify_bytes_openpgp(canonical_json, &eddsa_signature, &public_key)
        }
    }
}

fn parse_ed25519_signature(bytes: &[u8]) -> Result<EdDSASignature, Error> {
    let bytes: &[u8; 64] = bytes.try_into().map_err(|_| Error::InvalidSignatureValue)?;
    if !is_canonical_ed25519_scalar(&bytes[32..]) {
        return Err(Error::InvalidSignatureValue);
    }

    Ok(EdDSASignature::from_bytes(bytes))
}

fn is_canonical_ed25519_scalar(scalar: &[u8]) -> bool {
    scalar
        .iter()
        .rev()
        .zip(ED25519_SCALAR_ORDER.iter().rev())
        .find_map(|(byte, limit)| (byte != limit).then_some(byte < limit))
        .unwrap_or(false)
}

// OpenPGP uses the hash of the message as input to the signature algorithm
// https://datatracker.ietf.org/doc/html/rfc4880#section-5.2.4
// This is used when signing with OpenPGP application on smartcards
fn verify_bytes_openpgp(
    bytes: impl AsRef<[u8]>,
    signature: &EdDSASignature,
    public_key: &VerifyingKey,
) -> Result<(), Error> {
    let bytes_hash = create_digest(bytes, &HashAlgorithm::Sha512);
    verify_bytes(bytes_hash, signature, public_key)
}

fn verify_bytes(
    bytes: impl AsRef<[u8]>,
    signature: &EdDSASignature,
    public_key: &VerifyingKey,
) -> Result<(), Error> {
    public_key
        .verify_strict(bytes.as_ref(), signature)
        .map_err(|_| Error::InvalidSignature)
}

fn validate_ed25519_key(key: &Key) -> Result<(), Error> {
    let is_ed25519 = matches!(key.algorithm, EncryptionAlgorithm::EdDSA)
        && key
            .parameters
            .as_ref()
            .and_then(|parameters| parameters.get("scheme"))
            .and_then(Value::as_str)
            == Some("Ed25519");

    if is_ed25519 {
        Ok(())
    } else {
        Err(Error::UnsupportedPublicKeyAlgorithm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_public_key() -> Key {
        let mut key = [0u8; 32];
        key[0] = 1;
        key.into()
    }

    fn universal_signature() -> [u8; 64] {
        let mut signature = [0x66; 64];
        signature[0] = 0x58;
        signature[32] = 1;
        signature[33..].fill(0);
        signature
    }

    #[test]
    fn rejects_identity_public_key() {
        let value = json!({ "role": "admin" });
        let public_key = identity_public_key();

        for encryption in [
            EncryptionAlgorithm::EdDSA,
            EncryptionAlgorithm::EdDSAOpenPGP,
        ] {
            let algorithm = SignatureAlgorithm {
                hash: HashAlgorithm::Sha512,
                encryption,
            };

            assert_eq!(
                verify_signature_json(&value, &algorithm, universal_signature(), &public_key),
                Err(Error::InvalidSignature)
            );
        }
    }

    #[test]
    fn rejects_unsupported_signature_hash() {
        let value = json!({ "role": "admin" });
        let key_pair = create_key_pair();
        let (_, signature) = sign_json(&value, &key_pair.private_key).unwrap();
        let algorithm = SignatureAlgorithm {
            hash: HashAlgorithm::Sha256,
            encryption: EncryptionAlgorithm::EdDSA,
        };

        assert_eq!(
            verify_signature_json(&value, &algorithm, signature, &key_pair.public_key),
            Err(Error::UnsupportedSignatureAlgorithm)
        );
    }

    #[test]
    fn rejects_unsupported_key_metadata() {
        let value = json!({ "role": "admin" });
        let key_pair = create_key_pair();
        let (algorithm, signature) = sign_json(&value, &key_pair.private_key).unwrap();
        let mut public_key = key_pair.public_key;
        public_key.parameters = Some(json!({ "scheme": "NotEd25519" }));

        assert_eq!(
            verify_signature_json(&value, &algorithm, signature, &public_key),
            Err(Error::UnsupportedPublicKeyAlgorithm)
        );
    }
}
