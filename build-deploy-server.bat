cargo build --locked --release --bin backups-server
mkdir live\server
move target\release\backups-server.exe live\server\backups-server.exe
cd live\server
start backups-server.exe
