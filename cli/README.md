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
