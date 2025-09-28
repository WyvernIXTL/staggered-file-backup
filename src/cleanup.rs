// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

use color_eyre::eyre::{Ok, Result};
use log::warn;

type Entry = ((u32, u32, u32), PathBuf);

fn compare_entries(a: &Entry, b: &Entry) -> Ordering {
    match a.0.0.cmp(&b.0.0) {
        Ordering::Equal => match a.0.1.cmp(&b.0.1) {
            Ordering::Equal => a.0.2.cmp(&b.0.2),
            other => other,
        },
        other => other,
    }
}

fn identify_files_to_keep(
    file_list: &Vec<Entry>,
    keep_latest: Option<u32>,
    keep_daily: Option<u32>,
    keep_monthly: Option<u32>,
    keep_yearly: Option<u32>,
) -> Result<Vec<((u32, u32, u32), PathBuf)>> {
    if file_list.is_empty() {
        warn!("No files are backed up! Cleanup skipped.");
        return Ok(vec![]);
    }

    let mut file_list = file_list.clone();
    file_list.sort_by(compare_entries);
    let file_list = file_list;

    let mut keep = vec![];

    if let Some(keep_latest) = keep_latest {
        let keep_latest = usize::try_from(keep_latest)?;
        let start_index = file_list.len() - keep_latest;
        keep.extend_from_slice(&file_list[start_index..]);
    }

    if let Some(keep_daily) = keep_daily {
        let mut filtered = vec![];
        filtered.push(file_list.first().unwrap());
        for file in file_list.iter() {
            if filtered.last().unwrap().0 != file.0 {
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
            if filtered.last().unwrap().0.0 != file.0.0 || filtered.last().unwrap().0.1 != file.0.1
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
            if filtered.last().unwrap().0.0 != file.0.0 {
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

    keep_dedup.sort_by(compare_entries);

    Ok(keep_dedup)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ordering() {
        let path = PathBuf::new();
        let mut entries = vec![
            ((2025, 08, 01), path.clone()),
            ((2025, 09, 01), path.clone()),
            ((2023, 08, 01), path.clone()),
            ((2025, 08, 02), path.clone()),
        ];

        entries.sort_by(compare_entries);

        assert_eq!(
            entries,
            vec![
                ((2023, 8, 1), path.clone()),
                ((2025, 8, 1), path.clone()),
                ((2025, 8, 2), path.clone()),
                ((2025, 9, 1), path.clone()),
            ]
        )
    }

    #[test]
    fn test_files_to_keep_latest() {
        let files = vec![
            ((2025, 08, 01), PathBuf::from("a")),
            ((2025, 09, 01), PathBuf::from("b")),
            ((2025, 10, 01), PathBuf::from("c")),
            ((2025, 10, 02), PathBuf::from("e")),
            ((2025, 10, 01), PathBuf::from("d")),
            ((2025, 09, 02), PathBuf::from("f")),
            ((2023, 08, 01), PathBuf::from("g")),
            ((2025, 08, 02), PathBuf::from("h")),
        ];

        assert_eq!(
            identify_files_to_keep(&files, Some(3), None, None, None).unwrap(),
            vec![
                ((2025, 10, 01), PathBuf::from("c")),
                ((2025, 10, 01), PathBuf::from("d")),
                ((2025, 10, 02), PathBuf::from("e"))
            ]
        )
    }

    #[test]
    fn test_files_to_keep_daily() {
        let files = vec![
            ((2025, 08, 01), PathBuf::from("a")),
            ((2025, 09, 01), PathBuf::from("b")),
            ((2025, 10, 01), PathBuf::from("c")),
            ((2025, 10, 02), PathBuf::from("e")),
            ((2025, 10, 01), PathBuf::from("d")),
            ((2025, 09, 02), PathBuf::from("f")),
            ((2023, 08, 01), PathBuf::from("g")),
            ((2025, 08, 02), PathBuf::from("h")),
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, Some(4), None, None).unwrap(),
            vec![
                ((2025, 09, 01), PathBuf::from("b")),
                ((2025, 09, 02), PathBuf::from("f")),
                ((2025, 10, 01), PathBuf::from("c")),
                ((2025, 10, 02), PathBuf::from("e"))
            ]
        )
    }

    #[test]
    fn test_files_to_keep_monthly() {
        let files = vec![
            ((2025, 08, 01), PathBuf::from("a")),
            ((2025, 09, 01), PathBuf::from("b")),
            ((2025, 10, 01), PathBuf::from("c")),
            ((2025, 10, 02), PathBuf::from("e")),
            ((2025, 10, 01), PathBuf::from("d")),
            ((2025, 09, 02), PathBuf::from("f")),
            ((2023, 08, 01), PathBuf::from("g")),
            ((2025, 08, 02), PathBuf::from("h")),
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, None, Some(3), None).unwrap(),
            vec![
                ((2025, 08, 01), PathBuf::from("a")),
                ((2025, 09, 01), PathBuf::from("b")),
                ((2025, 10, 01), PathBuf::from("c")),
            ]
        )
    }

    #[test]
    fn test_files_to_keep_yearly() {
        let files = vec![
            ((2025, 08, 01), PathBuf::from("a")),
            ((2025, 09, 01), PathBuf::from("b")),
            ((2025, 10, 01), PathBuf::from("c")),
            ((2025, 10, 02), PathBuf::from("e")),
            ((2025, 10, 01), PathBuf::from("d")),
            ((2025, 09, 02), PathBuf::from("f")),
            ((2023, 08, 01), PathBuf::from("g")),
            ((2025, 08, 02), PathBuf::from("h")),
        ];

        assert_eq!(
            identify_files_to_keep(&files, None, None, None, Some(2)).unwrap(),
            vec![
                ((2023, 08, 01), PathBuf::from("g")),
                ((2025, 08, 01), PathBuf::from("a")),
            ]
        )
    }
}
