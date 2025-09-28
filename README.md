<div align="center">

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

Please only use the directory for this single backup and nothing else!

## Performance

Currently the project is not optimized.
PGO + LTO make no difference as the bottleneck is with the system (and with my bad code).
