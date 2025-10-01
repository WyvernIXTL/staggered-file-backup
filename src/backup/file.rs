// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{OsStr, OsString},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use color_eyre::{
    Section,
    eyre::{Context, ContextCompat, Ok, Result},
};

use crate::{backup::cleanup::BackupFile, backup::parsing::metadata_from_directory};

fn modified_date<P>(file_path: P) -> Result<SystemTime>
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();
    let metadata = path.metadata()?;
    metadata
        .modified()
        .wrap_err("Failed getting metadata of source file.")
        .suggestion("Was this program executed with permissions to read the input file?")
}

fn unix_from_system_time(time: SystemTime) -> Result<Duration> {
    time.duration_since(UNIX_EPOCH)
        .wrap_err("Failed converting system time to unix time.")
        .note("This should never happen D:")
}

fn date_string_from_seconds(time_in_seconds: i64) -> Result<String> {
    let date = DateTime::from_timestamp_secs(time_in_seconds)
        .wrap_err("Failed to parse date in seconds to date time.")?;

    Ok(date.format("%F").to_string())
}

pub fn modified_date_string_from_path<P>(file_path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let modified = modified_date(file_path)?;
    let duration = unix_from_system_time(modified)?;
    let seconds = i64::try_from(duration.as_secs())?;
    date_string_from_seconds(seconds)
}

pub fn target_file_name(
    target_dir: impl AsRef<Path>,
    modified_date: impl AsRef<str>,
    base_name: impl AsRef<OsStr>,
    extension: Option<impl AsRef<OsStr>>,
) -> Result<OsString> {
    let mut file_name = OsString::new();
    file_name.push(modified_date.as_ref());

    //TODO: This is terrible.
    let mut backup_files: Vec<BackupFile> = metadata_from_directory(target_dir.as_ref())?
        .into_iter()
        .filter(|file| {
            file.path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(&modified_date.as_ref())
        })
        .collect();

    backup_files.sort();

    let counter = backup_files
        .last()
        .map_or(0, |file| file.metadata.counter + 1);
    let mut file_name = file_name.clone();
    file_name.push(format!("_{:>02}_", counter));
    file_name.push(base_name.as_ref());
    if let Some(ext) = extension.as_ref() {
        file_name.push(".");
        file_name.push(ext.as_ref());
    }
    Ok(file_name)
}
