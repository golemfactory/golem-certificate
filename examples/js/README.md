To use this example install dependencies via `npm install`.

Verifying a certificate signature (just the signature, not the certificate chain) can be accomplished by running the program:  
`node . certificate-path` where certificate path is the path to a Golem certificate json.

Example:  
`node . ../../tests/resources/certificate/happy_path.signed.json` - this should print that the signature is valid  
`node . ../../tests/resources/certificate/invalid_signature.signed.json` - this should print that the verification failed
