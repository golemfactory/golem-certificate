use anyhow::Result;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};

use ed25519_dalek::{
    ExpandedSecretKey, Keypair, PublicKey, SecretKey, Signature as EdDSASignature, Verifier,
};
use rand::rngs::OsRng;

use crate::schemas::signature::SignatureAlgorithm;
use crate::serde_jcs;
use crate::serde_utils::{bytes_to_hex, hex_to_bytes};
use crate::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HashAlgorithm {
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_224,
    Sha3_256,
    Sha3_384,
    Sha3_512,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha512
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncryptionAlgorithm {
    EdDSA,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Key {
    algorithm: EncryptionAlgorithm,
    #[serde(serialize_with = "bytes_to_hex", deserialize_with = "hex_to_bytes")]
    key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Value>,
}

pub struct KeyPair {
    pub public_key: Key,
    pub private_key: Key,
}

pub fn create_key_pair() -> KeyPair {
    let mut csprng = OsRng {};
    let keypair = Keypair::generate(&mut csprng);
    let public_key = Key {
        algorithm: EncryptionAlgorithm::EdDSA,
        parameters: Some(json!({ "scheme": "Ed25519" })),
        key: keypair.public.to_bytes().into(),
    };
    let private_key = Key {
        algorithm: EncryptionAlgorithm::EdDSA,
        parameters: Some(json!({ "scheme": "Ed25519" })),
        key: keypair.secret.to_bytes().into(),
    };
    KeyPair {
        public_key,
        private_key,
    }
}

pub fn create_default_hash(value: &Value) -> Result<Vec<u8>, Error> {
    create_hash(value, &HashAlgorithm::default())
}

pub fn create_hash(value: &Value, hash_algorithm: &HashAlgorithm) -> Result<Vec<u8>, Error> {
    let canonical_json =
        serde_jcs::to_vec(value).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    Ok(create_digest(canonical_json, hash_algorithm))
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
    let canonical_json = serde_jcs::to_vec(value)?;
    let secret_key = SecretKey::from_bytes(&private_key.key)?;
    let signature_value = sign_bytes(canonical_json, &secret_key);
    let algorithm = SignatureAlgorithm {
        hash: HashAlgorithm::Sha512,
        encryption: EncryptionAlgorithm::EdDSA,
    };
    Ok((algorithm, signature_value))
}

fn sign_bytes(bytes: impl AsRef<[u8]>, secret_key: &SecretKey) -> Vec<u8> {
    let expanded_secret_key = ExpandedSecretKey::from(secret_key);
    let public_key = PublicKey::from(secret_key);
    let signature_value = expanded_secret_key.sign(bytes.as_ref(), &public_key);
    signature_value.to_bytes().into()
}

pub fn verify_signature_json(
    value: &Value,
    signature_value: impl AsRef<[u8]>,
    public_key: &Key,
) -> Result<(), Error> {
    let canonical_json =
        serde_jcs::to_vec(value).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    let eddsa_signature = EdDSASignature::from_bytes(signature_value.as_ref())
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;
    let public_key =
        PublicKey::from_bytes(&public_key.key).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    verify_bytes(canonical_json, &eddsa_signature, &public_key)
}

fn verify_bytes(
    bytes: impl AsRef<[u8]>,
    signature: &EdDSASignature,
    public_key: &PublicKey,
) -> Result<(), Error> {
    public_key
        .verify(bytes.as_ref(), signature)
        .map_err(|_| Error::InvalidSignature)
}
