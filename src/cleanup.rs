// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp::Ordering, path::PathBuf};

use color_eyre::eyre::{Ok, Result};
use log::warn;

use crate::parsing::FileNameMetadata;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BackupFile {
    pub metadata: FileNameMetadata,
    pub path: PathBuf,
}

impl Ord for BackupFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.metadata.cmp(&other.metadata)
    }
}

impl PartialOrd for BackupFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn identify_files_to_keep(
    file_list: &Vec<BackupFile>,
    keep_latest: Option<u32>,
    keep_daily: Option<u32>,
    keep_monthly: Option<u32>,
    keep_yearly: Option<u32>,
) -> Result<Vec<BackupFile>> {
    if file_list.is_empty() {
        warn!("No files are backed up! Cleanup skipped.");
        return Ok(vec![]);
    }

    let mut file_list = file_list.clone();
    file_list.sort();
    let file_list = file_list;

    let mut keep = vec![];

    if let Some(keep_latest) = keep_latest {
        let keep_latest = usize::try_from(keep_latest)?;
        let start_index = if file_list.len() >= keep_latest {
            file_list.len() - keep_latest
        } else {
            0
        };
        keep.extend_from_slice(&file_list[start_index..]);
    }

    if let Some(keep_daily) = keep_daily {
        let mut filtered = vec![];
        filtered.push(file_list.first().unwrap());
        for file in file_list.iter() {
            if filtered.last().unwrap().metadata.year != file.metadata.year
                || filtered.last().unwrap().metadata.month != file.metadata.month
                || filtered.last().unwrap().metadata.day != file.metadata.day
            {
                filtered.push(file);
            }
        }

        let mut count = 0;
        while let Some(file) = filtered.pop() {
            if count == keep_daily {
                break;
            }

            keep.push(file.clone());
            count += 1;
        }
    }

    if let Some(keep_monthly) = keep_monthly {
        let mut filtered = vec![];
        filtered.push(file_list.first().unwrap());
        for file in file_list.iter() {
            if filtered.last().unwrap().metadata.year != file.metadata.year
                || filtered.last().unwrap().metadata.month != file.metadata.month
            {
                filtered.push(file);
            }
        }

        let mut count = 0;
        while let Some(file) = filtered.pop() {
            if count == keep_monthly {
                break;
            }

            keep.push(file.clone());
            count += 1;
        }
    }

    if let Some(keep_yearly) = keep_yearly {
        let mut filtered = vec![];
        filtered.push(file_list.first().unwrap());
        for file in file_list.iter() {
            if filtered.last().unwrap().metadata.year != file.metadata.year {
                filtered.push(file);
            }
        }

        let mut count = 0;
        while let Some(file) = filtered.pop() {
            if count == keep_yearly {
                break;
            }

            keep.push(file.clone());
            count += 1;
        }
    }

    let mut keep_dedup = vec![];
    for file in keep.into_iter() {
        if !keep_dedup.contains(&file) {
            keep_dedup.push(file);
        }
    }

    keep_dedup.sort();

    Ok(keep_dedup)
}

pub fn identify_files_to_delete(
    file_list: Vec<BackupFile>,
    files_to_keep: &Vec<BackupFile>,
) -> Vec<BackupFile> {
    file_list
        .into_iter()
        .filter(|file| !files_to_keep.contains(file))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parsing::FileNameMetadata;

    #[test]
    fn test_files_to_keep_latest() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        assert_eq!(
            identify_files_to_keep(&files, Some(3), None, None, None).unwrap(),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("c"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 2
                    },
                    path: PathBuf::from("d"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("e"),
                }
            ]
        )
    }

    #[test]
    fn test_files_to_keep_daily() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, Some(4), None, None).unwrap(),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("b"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("f"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("c"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("e"),
                }
            ]
        )
    }

    #[test]
    fn test_files_to_keep_monthly() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, None, Some(3), None).unwrap(),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 8,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("a"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("b"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("c"),
                },
            ]
        )
    }

    #[test]
    fn test_files_to_keep_yearly() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, None, None, Some(2)).unwrap(),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2023,
                        month: 8,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("g"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 8,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("a"),
                },
            ]
        )
    }

    #[test]
    fn test_files_to_keep_multi() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        assert_eq!(
            identify_files_to_keep(&files, Some(3), Some(4), Some(3), Some(2)).unwrap(),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2023,
                        month: 8,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("g"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 8,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("a"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("b"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("f"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("c"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 2
                    },
                    path: PathBuf::from("d"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("e"),
                },
            ]
        )
    }

    #[test]
    fn test_files_to_delete() {
        let files = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("b"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("c"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 9,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("f"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("h"),
            },
        ];

        let keep = vec![
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2023,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("g"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 8,
                    day: 1,
                    counter: 1,
                },
                path: PathBuf::from("a"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 1,
                    counter: 2,
                },
                path: PathBuf::from("d"),
            },
            BackupFile {
                metadata: FileNameMetadata {
                    year: 2025,
                    month: 10,
                    day: 2,
                    counter: 1,
                },
                path: PathBuf::from("e"),
            },
        ];

        assert_eq!(
            identify_files_to_delete(files, &keep),
            vec![
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("b"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 10,
                        day: 1,
                        counter: 1
                    },
                    path: PathBuf::from("c"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 9,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("f"),
                },
                BackupFile {
                    metadata: FileNameMetadata {
                        year: 2025,
                        month: 8,
                        day: 2,
                        counter: 1
                    },
                    path: PathBuf::from("h"),
                },
            ]
        );
    }
}
