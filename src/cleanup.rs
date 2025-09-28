// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp::Ordering, path::PathBuf};

use color_eyre::eyre::Result;

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
    let mut file_list = file_list.clone();
    file_list.sort_by(compare_entries);
    let file_list = file_list;

    let mut keep = vec![];

    if let Some(keep_latest) = keep_latest {
        let keep_latest = usize::try_from(keep_latest)?;
        let start_index = file_list.len() - keep_latest;
        keep.extend_from_slice(&file_list[start_index..]);
    }

    todo!()
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
}
