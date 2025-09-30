<div align="center">

<img src="./icon/Icon-exported-no-crop-p1.svg" height=256 />

# `staggered-file-backup`

**An easy and secure staggered file backup solution**

</div>

> [!WARNING]
> Work in Progress!

`staggered-file-backup` is designed to backup a single file!

## Usage

To backup a single file to a directory:

```sh
staggered-file-backup ./path/to/source/file ./path/to/target/backup/dir/
```

## Installation

### Compiled Binaries

[![Download for Windows](https://img.shields.io/badge/Download-Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)![Download for macOS](https://img.shields.io/badge/Download-macOS-000000?style=for-the-badge&logo=apple&logoColor=white)![Download for Linux](https://img.shields.io/badge/Download-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/WyvernIXTL/staggered-file-backup/releases/latest)

### From Source

```sh
cargo install staggered-file-backup
```

Please only use the directory for this single backup and nothing else!

## Performance

Currently the project is not optimized.
PGO + LTO make no difference as the bottleneck is with the system (and with my bad code).
