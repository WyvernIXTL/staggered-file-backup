param(
    [string]$TargetTriple
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

$bin = "staggered-file-backup"

$CargoToml = Get-Content ./Cargo.toml -Raw

$regexPattern = '^version\s?\=\s?\"(\d{1,2}\.\d{1,2}\.\d{1,2}.*?)\"$'
$regexOptions = [System.Text.RegularExpressions.RegexOptions]::Multiline
$regex = New-Object System.Text.RegularExpressions.Regex($regexPattern, $regexOptions)

$match = $regex.Match($CargoToml)

if (-not $match.Success) {
    throw "Failed to regex version"
}
if ($match.Groups.Count -eq 0) {
    throw "Match found but no capture group available"
}

$version = $match.Groups[1].Value

$ArchiveName = "$bin-v$version-$TargetTriple"

if ($IsWindows) {
    7z a ./target/$ArchiveName.7z ./README.md ./LICENSE ./target/$TargetTriple/optimized/$bin.exe ./CHANGELOG.md
} else {
    tar cfJ ./target/$ArchiveName.tar.xz ./README.md ./LICENSE ./target/$TargetTriple/optimized/$bin ./CHANGELOG.md
}
