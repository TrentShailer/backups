cargo build --locked --release --bin backups-client
mkdir -p ./live/client/
chmod a+x ./target/release/backups-client
mv ./target/release/backups-client ./live/client/backups-client
cd ./live/client
./backups-client &
