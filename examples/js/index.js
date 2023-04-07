const fs = require('fs');
const canonicalize = require('canonicalize');
const elliptic = require('elliptic');

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

const encoder = new TextEncoder();
const signed_bytes = encoder.encode(canonicalize(certificate.certificate)); // encode the string into bytes with UTF-8 encoding

const result = elliptic.eddsa('ed25519').verify(signed_bytes, certificate.signature.value, signing_certificate.certificate.publicKey.key);

if (result) {
    console.log("The signature is valid.");
} else {
    console.log("Signature verification failed.");
}
