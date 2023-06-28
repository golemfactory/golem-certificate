# Golem Certificate

Golem Certificate is a certificate format defined in [GAP-25](https://github.com/golemfactory/golem-architecture/blob/master/gaps/gap-25_golem_certificates/gap-25_golem_certificates.md). Node descriptors used in the Golem network to identify 'Requestor' agents are defined in [GAP-31](https://github.com/golemfactory/golem-architecture/blob/master/gaps/gap-31_node_descriptor/gap-31_node_descriptor.md). 
This library provides utility functions to work with Golem Certificates and Node descriptors and relies on the JSON schema files defined in the above mentioned GAPs.
The library currently only supports Ed25519 signature scheme and provides the following basic functions
- Create a keypair
- Sign a self-signed Golem Certificate
- Sign a Golem Certificate or a Node descriptor with a Golem Certificate
- Verify a variant of the Ed25519 signature where the hash of the message is fed into the signature algorithm instead of the full message. This is useful when using smartcards running OpenPGP to create signatures using the private key stored on the smartcard.

The `cli` directory contains a command line utility that demonstrates how to use the library, it also includes a terminal based UI that guides through the generation process of Golem Certificates and Node descriptors.
