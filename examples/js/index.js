const fs = require('fs');
const crypto = require('crypto');
const canonicalize = require('canonicalize');

const ED25519_SPKI_PREFIX = Buffer.from('302a300506032b6570032100', 'hex');

const filename = process.argv[process.argv.length - 1];
console.log("Reading certificate from file " + filename);

const certificate_data = fs.readFileSync(filename);
const certificate = JSON.parse(certificate_data);
const signing_certificate = certificate.signature.signer === "self" ? certificate : certificate.signature.signer;

if (certificate.signature.algorithm.hash !== "sha512"
    || certificate.signature.algorithm.encryption !== "EdDSA"
    || signing_certificate.certificate.publicKey.parameters.scheme !== "Ed25519") {
    console.log("Unsupported signature type");
    process.exit(1);
}

const signed_bytes = Buffer.from(canonicalize(certificate.certificate), 'utf8');
const public_key = crypto.createPublicKey({
    key: Buffer.concat([
        ED25519_SPKI_PREFIX,
        Buffer.from(signing_certificate.certificate.publicKey.key, 'hex'),
    ]),
    format: 'der',
    type: 'spki',
});

const result = crypto.verify(
    null,
    signed_bytes,
    public_key,
    Buffer.from(certificate.signature.value, 'hex'),
);

if (result) {
    console.log("The signature is valid.");
} else {
    console.log("Signature verification failed.");
}
