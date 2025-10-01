New-Item -Path ./target/test-db/ -Type Directory  -ErrorAction SilentlyContinue
$env:DATABASE_URL="./target/test-db/test.sqlite3"
