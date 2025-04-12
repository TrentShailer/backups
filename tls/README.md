# TLS

## Generate CA

`cfssl gencert -config config.json -profile authority -initca csr_ca.json | cfssljson -bare ca`

## Generate Clients

* `cfssl gencert -ca ca.pem -ca-key ca-key.pem -config config.json -profile client csr_sender.json | cfssljson -bare sender`
* `cfssl gencert -ca ca.pem -ca-key ca-key.pem -config config.json -profile client csr_receiver.json | cfssljson -bare receiver`

## Info

* `openssl x509 -in ca.pem -text`
