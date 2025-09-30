param(
    [string]$TargetTriple
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"


$bin = "staggered-file-backup"

New-Item -Type Directory -Path ./test/ -ErrorAction SilentlyContinue
New-Item -Type Directory -Path ./test/target/ -ErrorAction SilentlyContinue

$content = "This is test data for a large file. " * 1000000
$content | Out-File -FilePath "./test/largefile.txt" -Encoding UTF8

# Compile with instrumentation:
# cargo pgo instrument build -- --profile optimized --target x86_64-pc-windows-msvc

$env:LLVM_PROFILE_FILE="./target/pgo-profiles/$($bin)_%m_%p.profraw"

& ./target/$TargetTriple/optimized/$bin ./test/largefile.txt ./test/target/
& ./target/$TargetTriple/optimized/$bin ./test/largefile.txt ./test/target/
& ./target/$TargetTriple/optimized/$bin ./test/largefile.txt ./test/target/
& ./target/$TargetTriple/optimized/$bin --help
& ./target/$TargetTriple/optimized/$bin --licenses
& ./target/$TargetTriple/optimized/$bin -n 3 -m 2 ./test/largefile.txt ./test/target/
& ./target/$TargetTriple/optimized/$bin -y 1 -d 1 ./test/largefile.txt ./test/target/

# Compile optimized:
# cargo pgo optimize build -- --profile optimized --target x86_64-pc-windows-msvc
