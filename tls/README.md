# TLS

## Generate Certificates

```powershell
cfssl gencert -config config.json -profile authority -initca csr_ca.json | cfssljson -bare backup-ca
cfssl gencert -ca backup-ca.pem -ca-key backup-ca-key.pem -config config.json -profile client csr_sender.json | cfssljson -bare backup-sender
cfssl gencert -ca backup-ca.pem -ca-key backup-ca-key.pem -config config.json -profile client csr_receiver.json | cfssljson -bare backup-receiver
```

## Renew CA

`cfssl gencert -renewca -ca backup-ca.pem -ca-key backup-ca-key.pem`

## Info

* `openssl x509 -in backup-ca.pem -text`
