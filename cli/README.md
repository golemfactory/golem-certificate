# Golem certificate command line utility

## Golem Certificate Manager

Golem Certificate Manager is a terminal utility that helps in checking the details of signed documents (certificates or node descriptors) and guides through the document creation and signing process.

It is controlled with the keyboard:
- the arrow keys used to move around in menus, scroll verification views and to move the select the active part in the editor
- Enter key is used to interact with highlighted elements
- Esc key is used to go back to the previous component without saving changes
- Ctrl-C terminates the program immediately and without saving any changes

In the file browser the Backspace key can be used to move up to the parent directory (if there is any).

In Save file dialog the Tab key is used to change between the active panels of save file dialog. 

## Command line commands

The utility can be used from the command line by advanced users. These commands expect the user to know exactly what they are doing and do little to no verification.

### create-key-pair

The `create-key-pair` command will create a keypair and save two files on the path specified by the <KEY_PAIR_PATH> parameter. The public key will have an extension set to `.pub.json` while the private key file's extension is set to `.key.json`

### fingerprint

This command generates the Sha512 fingerprint of the signed part of a golem certificate or node descriptor file. The input is required to be a JSON file that defines the `$schema` property based on which the utility will get the signed part of the JSON. This command does not do any kind of verification.

### self-sign-certificate

Facilitates creating a self signed certificate. Arguments are:
- <CERTIFICATE_PATH> the JSON file containing a certificate structure. The JSON has to have the `$schema` property set correctly and the `certificate` property with the appropriate content according to the schema. Other parts of the JSON are ignored and the `signature` property is overwritten if there was any.
- <PRIVATE_KEY_PATH> the JSON file containing the private key corresponding to the public key defined in the certificate, during the signing process there is no verification of any kind, the user is responsible for providing the correct signing key.

The output is saved on the same path as the input certificate with the extension set to `.signed.json`. Any existing file on that path will be overwritten.

### sign

Similar to creating self signed certificates, this command allows signing golem certificates and also node descriptors. Arguments are:
- <INPUT_FILE_PATH> the JSON file containing a certificate or node descriptor structure. The JSON has to have the `$schema` property set correctly and the `certificate` or `nodeDescriptor` property with the appropriate content according to the schema. Other parts of the JSON are ignored and the `signature` property is overwritten if there was any.
- <CERTIFICATE_PATH> the path pointing to a signed golem certificate to be used as signing certificate.
- <PRIVATE_KEY_PATH> the JSON file containing the private key corresponding to the public key defined in the signing certificate, during the signing process there is no verification of any kind, the user is responsible for providing the correct signing key.

The output is saved on the input file path with the extension set to `.signed.json`. Any existing file ont hat path will be overwritten.

### verify

The command allows verification of certificate or node descriptor JSON files. Apart from the document to be verified an optional timestamp in RFC 3339 format (ex: 2020-01-01T13:42:32Z) can be provided or 'now' to refer to the current time. If the timestamp is provided the document will be verified (beside all other verification) if it is valid at the point of time (the timestamp is within the validity period of the document).

## Smartcard Support

The CLI can utilize smartcards that support OpenPGP with ed25519 signature scheme. This capability is enabled with the `smartcard` feature.
On Linux this requires `pkg-config`, `libpcsclite-dev` to compile and `libpcsclite` to run. You will need to use some other software to generate the appropriate signing OpenPGP key on the device or migrate a generated key to a device. If a key is uploaded to the device, make sure that the public key is also accessible on the device as some devices might not be able to calculate the public key and it is verified during sign operations. The existence of the public key can be verified via the `export-public-key` command.

## Smartcard commands

The smartcard enabled sub menu is accessible via the `smartcard` command. The following subcommands exists:

### list

Lists all the smartcards that it can find that run the OpenPGP application. If your smartcard is plugged in but not listed here, try to remove the device and plug in again as it can happen if some other program is still using the smartcard.
All other subcommands require to specify the `ident` string of the card as listed by this command.

### export-public-key

In case you need the public key of the signing key from the card, this subcommand will save it in a JSON file.

### sign

This subcommands allows to sign golem certificates and node descriptors. The only different to the software key version is that instead of specifying the signing key file, the ident string of the smartcard is specified.
The signature logic will verify that the signing certificate has the same public key as the one exported from the card.

### self-sign-certificate

This subcommands self signs a certificate. Works similarly as the software key version, except that during creating the signature the public key in the input file is replaced with the one exported from the card to match the signing key.
