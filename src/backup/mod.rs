// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsString, path::PathBuf, process::exit};

use color_eyre::{
    Result, Section,
    eyre::{Context, ContextCompat},
};
use log::{error, info};

use crate::backup::{
    cleanup::{identify_files_to_delete, identify_files_to_keep},
    file::{modified_date_string_from_path, target_file_name},
    hash::{generate_sha256_file_content, hash_file},
    parsing::metadata_from_directory,
};

pub mod cleanup;
pub mod file;
pub mod hash;
pub mod parsing;

pub fn backup(
    source: PathBuf,
    target: PathBuf,
    keep_latest: Option<u32>,
    keep_daily: Option<u32>,
    keep_monthly: Option<u32>,
    keep_yearly: Option<u32>,
) -> Result<()> {
    info!("Source file path: {}", source.display());

    let source_basename = source
        .file_stem()
        .wrap_err("Failed extracting the basename (file stem) from source path.")?
        .to_os_string();
    info!("Source basename: {}", source_basename.display());

    let extension_option = source.extension().map(|ext| ext.to_os_string());
    match &extension_option {
        Some(ext) => info!("Source file extension: {}", ext.display()),
        None => log::warn!("Source file has no file extension."),
    }

    info!("Reading modification date of source file.");
    let modified_string = modified_date_string_from_path(&source)?;
    info!("Source file last modified: {}", &modified_string);

    info!("Hashing source file.");
    let source_hash = hash_file(&source)?;
    info!("Source file sh256: {}", &source_hash);

    info!("Target directory: {}", target.display());

    let target_file = target_file_name(
        &target,
        &modified_string,
        &source_basename,
        extension_option,
    )?;

    info!("Target file: {}", target_file.display());

    let target_file_path = target.join(&target_file);
    info!("Target file path: {}", target_file_path.display());

    info!(
        "Copying file '{}' to '{}'",
        source.display(),
        target_file_path.display()
    );

    std::fs::copy(source, &target_file_path)
        .wrap_err("Failed to copy source file to target dir.")
        .suggestion("Check if the target dir exists and if you have permissions to access it.")?;

    info!("Finished copying.");

    info!("Hashing target file.");
    let target_hash = hash_file(&target_file_path)?;
    info!("Target file sh256: {}", &target_hash);

    if target_hash == source_hash {
        info!("Target and source file hash are equal.");
    } else {
        error!("Target and source file hash are NOT equal! Exiting...");
        exit(1);
    }

    let mut hash_file_name = OsString::from(&target_file);
    hash_file_name.push(".sha256");
    let hash_file_path = &target.join(hash_file_name);

    info!("Write hash to file: {}", hash_file_path.display());

    std::fs::write(
        hash_file_path,
        generate_sha256_file_content(source_hash, target_file.to_string_lossy()),
    )
    .wrap_err("Failed to write hash file.")?;
    info!("Write success!");

    info!("Starting cleanup.");

    info!("Parsing files of target directory for dates.");
    let backup_files = metadata_from_directory(&target)?;

    info!("Determine which files to keep...");

    let backup_files_to_keep = identify_files_to_keep(
        &backup_files,
        keep_latest,
        keep_daily,
        keep_monthly,
        keep_yearly,
    )
    .wrap_err("Failed to determine which files to keep.")?;

    backup_files_to_keep
        .iter()
        .for_each(|file| info!("KEEP: {}", file.path.display()));

    info!("Determine which files to move into recycle bin...");
    let files_to_trash = identify_files_to_delete(backup_files, &backup_files_to_keep);

    files_to_trash
        .iter()
        .for_each(|file| info!("TRASH: {}", file.path.display()));

    let files_to_trash_count = files_to_trash.len();
    let mut files_to_trash_paths: Vec<PathBuf> =
        files_to_trash.into_iter().map(|file| file.path).collect();
    let files_to_trash_paths_sum_files: Vec<PathBuf> = files_to_trash_paths
        .iter()
        .map(|path| {
            let mut path_str = path.clone().into_os_string();
            path_str.push(".sha256");
            PathBuf::from(path_str)
        })
        .collect();
    files_to_trash_paths.extend_from_slice(&files_to_trash_paths_sum_files);

    if files_to_trash_count > 0 {
        info!("Moving files into recycle bin...");
        trash::delete_all(files_to_trash_paths)?;

        info!("Moved {} files into recycle bin.", files_to_trash_count);
    } else {
        info!("No files where determined to be moved into recycle bin.");
    }

    info!("DONE!");

    return Ok(());
}
