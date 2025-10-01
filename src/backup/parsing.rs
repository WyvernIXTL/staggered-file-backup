// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cmp::Ordering;
use std::{ffi::OsStr, path::Path, sync::LazyLock};

use color_eyre::Result;
use color_eyre::eyre::{ContextCompat, Ok, ensure};
use log::{error, warn};
use regex::Regex;

use crate::backup::cleanup::BackupFile;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileNameMetadata {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub counter: u32,
}

impl Ord for FileNameMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.year.cmp(&other.year) {
            Ordering::Equal => match self.month.cmp(&other.month) {
                Ordering::Equal => match self.day.cmp(&other.day) {
                    Ordering::Equal => self.counter.cmp(&other.counter),
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}

impl PartialOrd for FileNameMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn metadata_from_file_name(file_name: impl AsRef<OsStr>) -> Option<FileNameMetadata> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?<year>\d{4})\-(?<month>\d{2})\-(?<day>\d{2})\_(?<counter>\d{2})\_.*$")
            .expect("Failed parsing regex")
    });

    let file_name_string = file_name.as_ref().to_string_lossy();

    let capture = REGEX.captures(&file_name_string)?;

    let year_str = capture.name("year")?.as_str();
    let month_str = capture.name("month")?.as_str();
    let day_str = capture.name("day")?.as_str();
    let counter_str = capture.name("counter")?.as_str();

    let year: u32 = year_str.parse().inspect_err(|err| error!("{}", err)).ok()?;
    let month: u32 = month_str
        .parse()
        .inspect_err(|err| error!("{}", err))
        .ok()?;
    let day: u32 = day_str.parse().inspect_err(|err| error!("{}", err)).ok()?;
    let counter: u32 = counter_str
        .parse()
        .inspect_err(|err| error!("{}", err))
        .ok()?;

    Some(FileNameMetadata {
        year,
        month,
        day,
        counter,
    })
}

fn metadata_from_path(file_path: impl AsRef<Path>) -> Result<FileNameMetadata> {
    ensure!(
        file_path.as_ref().is_file(),
        "Path given to be parsed is not a file."
    );

    let file_name = file_path
        .as_ref()
        .file_name()
        .wrap_err("Failed extracting file name from path")?;

    metadata_from_file_name(file_name).wrap_err("Failed parsing file name to date.")
}

pub fn metadata_from_directory(dir_path: impl AsRef<Path>) -> Result<Vec<BackupFile>> {
    Ok(std::fs::read_dir(dir_path.as_ref())?
        .filter_map(|dir_entry_result| {
            dir_entry_result
                .inspect_err(|errr| warn!("Error while reading directory entries: {}", errr))
                .ok()
        })
        .filter(|entry| {
            let entry_name = entry.file_name();
            match entry.metadata() {
                Err(err) => {
                    warn!(
                        "Failed to read metadata of entry {}: {}",
                        &entry_name.display(),
                        err
                    );
                    false
                }
                std::result::Result::Ok(metadata) => {
                    if metadata.is_file() {
                        true
                    } else {
                        warn!("{} is not a file!", entry_name.display());
                        false
                    }
                }
            }
        })
        .map(|entry| entry.path())
        //TODO: Make better.
        .filter(|path| path.extension().map_or(true, |ext| ext != "sha256"))
        .filter_map(|path| {
            let date = metadata_from_path(&path)
                .inspect_err(|err| {
                    warn!(
                        "Failed parsing date of file {} with error: {}",
                        &path.display(),
                        err
                    )
                })
                .ok()?;

            Some(BackupFile {
                metadata: date,
                path: path,
            })
        })
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_file_name_valid() {
        let file_name = "2025-09-27_03_file1.txt.sha256";

        let result = metadata_from_file_name(file_name);

        assert_eq!(
            result,
            Some(FileNameMetadata {
                year: 2025,
                month: 09,
                day: 27,
                counter: 03
            })
        )
    }

    #[test]
    fn test_parse_file_name_invalid() {
        let file_name = "23-09-27_file1.txt.sha256";

        let result = metadata_from_file_name(file_name);

        assert_eq!(result, None)
    }

    #[test]
    fn test_ordering() {
        let mut entries = vec![
            FileNameMetadata {
                year: 2025,
                month: 08,
                day: 01,
                counter: 02,
            },
            FileNameMetadata {
                year: 2025,
                month: 09,
                day: 01,
                counter: 00,
            },
            FileNameMetadata {
                year: 2025,
                month: 08,
                day: 01,
                counter: 01,
            },
            FileNameMetadata {
                year: 2025,
                month: 08,
                day: 02,
                counter: 03,
            },
        ];

        entries.sort();

        assert_eq!(
            entries,
            vec![
                FileNameMetadata {
                    year: 2025,
                    month: 08,
                    day: 01,
                    counter: 01,
                },
                FileNameMetadata {
                    year: 2025,
                    month: 08,
                    day: 01,
                    counter: 02,
                },
                FileNameMetadata {
                    year: 2025,
                    month: 08,
                    day: 02,
                    counter: 03,
                },
                FileNameMetadata {
                    year: 2025,
                    month: 09,
                    day: 01,
                    counter: 00,
                },
            ]
        )
    }
}
