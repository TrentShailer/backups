# TLS

## Generate CA

`cfssl gencert -config config.json -profile authority -initca csr_ca.json | cfssljson -bare backups-ca`

`cfssl gencert -renewca -ca backups-ca.pem -ca-key backups-ca-key.pem`

## Generate Clients

* `cfssl gencert -ca backups-ca.pem -ca-key backups-ca-key.pem -config config.json -profile client csr_sender.json | cfssljson -bare backups-sender`
* `cfssl gencert -ca backups-ca.pem -ca-key backups-ca-key.pem -config config.json -profile client csr_receiver.json | cfssljson -bare backups-receiver`

## Info

* `openssl x509 -in backups-ca.pem -text`
