// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsString, fs::write, path::PathBuf, process::exit, str::FromStr};

use clap::{CommandFactory, Parser, ValueEnum, ValueHint};
use clap_complete::Shell;
use color_eyre::{
    Section,
    eyre::{Context, ContextCompat, Ok, Result},
};
use file::modified_date_string_from_path;
use license_fetcher::read_package_list_from_out_dir;
use log::{error, info, warn};

use crate::{
    cleanup::{identify_files_to_delete, identify_files_to_keep},
    file::target_file_name,
    hash::{generate_sha256_file_content, hash_file},
    logging::setup_logging,
    parsing::dates_from_directory,
    setup::setup_hooks,
};

mod cleanup;
mod file;
mod hash;
mod logging;
mod parsing;
mod setup;

fn parse_str_to_source_pathbuf(s: &str) -> std::result::Result<PathBuf, String> {
    match PathBuf::from_str(s) {
        std::result::Result::Ok(path_buf) => {
            if path_buf.is_file() && path_buf.try_exists().map_err(|err| err.to_string())? {
                std::result::Result::Ok(path_buf)
            } else {
                Err("Source is not a file".to_owned())
            }
        }
        Err(_) => Err("Source is not a path".to_owned()),
    }
}

fn parse_str_to_target_pathbuf(s: &str) -> std::result::Result<PathBuf, String> {
    match PathBuf::from_str(s) {
        std::result::Result::Ok(path_buf) => {
            if path_buf.is_dir() && path_buf.try_exists().map_err(|err| err.to_string())? {
                std::result::Result::Ok(path_buf)
            } else {
                Err("Target folder path is not a directory".to_owned())
            }
        }
        Err(_) => Err("Target folder path is not a path".to_owned()),
    }
}

/// An easy and secure staggered file backup solution
#[derive(Parser, Debug)]
#[command(version, about, author)]
struct Cli {
    /// Path to file to be backed up
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath, value_parser = parse_str_to_source_pathbuf, requires = "target")]
    source: Option<PathBuf>,

    /// Path to folder to place backups in
    ///
    /// Please do not use the folder for anything else!
    #[arg(value_name = "TARGET_FOLDER", value_hint = ValueHint::DirPath, value_parser = parse_str_to_target_pathbuf)]
    target: Option<PathBuf>,

    /// Set retention period for the newest backups.
    ///
    /// Setting the retention to n implies that the last n backups are kept regardless.
    /// A value of -1 implies no cleanup.
    #[arg(short = 'n', long = "keep-newest", default_value_t = 8, value_parser = clap::value_parser!(i32).range(-1..))]
    keep_newest_count: i32,

    /// Set retention period for the daily backups.
    ///
    /// Setting the retention to n implies that the last n daily backups are kept.
    /// A value of -1 implies no cleanup.
    #[arg(short = 'd', long = "keep-daily", default_value_t = 32, value_parser = clap::value_parser!(i32).range(-1..))]
    keep_daily_count: i32,

    /// Set retention period for the monthly backups.
    ///
    /// Setting the retention to n implies that the last n monthly backups are kept.
    /// A value of -1 implies no cleanup.
    #[arg(short = 'm', long = "keep-monthly", default_value_t = 12, value_parser = clap::value_parser!(i32).range(-1..))]
    keep_monthly_count: i32,

    /// Set retention period for the yearly backups.
    ///
    /// Setting the retention to n implies that the last n yearly backups are kept.
    /// A value of -1 implies no cleanup.
    #[arg(short = 'y', long = "keep-yearly", default_value_t = -1, value_parser = clap::value_parser!(i32).range(-1..))]
    keep_yearly_count: i32,

    /// Print licenses
    ///
    /// Print licenses of this project and all its dependencies
    #[arg(long, exclusive = true)]
    licenses: bool,

    /// List supported shells for shell completions
    #[arg(long, exclusive = true)]
    supported_shells: bool,

    /// Print shell completion for requested shell
    #[arg(long, exclusive = true, value_enum)]
    generate_completion: Option<Shell>,
}

fn main() -> Result<()> {
    setup_hooks()?;
    setup_logging()?;

    let cli = Cli::parse();

    if cli.licenses {
        let package_list = read_package_list_from_out_dir!()?;
        println!("{}", package_list);
        return Ok(());
    }

    if cli.supported_shells {
        for shell in Shell::value_variants() {
            println!("{shell}");
        }
        return Ok(());
    }

    if let Some(shell) = cli.generate_completion {
        let mut command = Cli::command();
        let command_name = command.get_name().to_string();
        eprintln!("Generating shell completions for {}", shell);
        clap_complete::generate(shell, &mut command, command_name, &mut std::io::stdout());
        return Ok(());
    }

    if let (Some(source_path), Some(target_dir_path)) = (cli.source, cli.target) {
        info!("Source file path: {}", source_path.display());

        let source_basename = source_path
            .file_stem()
            .wrap_err("Failed extracting the basename (file stem) from source path.")?
            .to_os_string();
        info!("Source basename: {}", source_basename.display());

        let extension_option = source_path.extension().map(|ext| ext.to_os_string());
        match &extension_option {
            Some(ext) => info!("Source file extension: {}", ext.display()),
            None => warn!("Source file has no file extension."),
        }

        info!("Reading modification date of source file.");
        let modified_string = modified_date_string_from_path(&source_path)?;
        info!("Source file last modified: {}", &modified_string);

        info!("Hashing source file.");
        let source_hash = hash_file(&source_path)?;
        info!("Source file sh256: {}", &source_hash);

        info!("Target directory: {}", target_dir_path.display());

        let target_file = target_file_name(
            &target_dir_path,
            &modified_string,
            &source_basename,
            extension_option,
        )?;

        info!("Target file: {}", target_file.display());

        let target_file_path = target_dir_path.join(&target_file);
        info!("Target file path: {}", target_file_path.display());

        info!(
            "Copying file '{}' to '{}'",
            source_path.display(),
            target_file_path.display()
        );

        std::fs::copy(source_path, &target_file_path)
            .wrap_err("Failed to copy source file to target dir.")
            .suggestion(
                "Check if the target dir exists and if you have permissions to access it.",
            )?;

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
        let hash_file_path = &target_dir_path.join(hash_file_name);

        info!("Write hash to file: {}", hash_file_path.display());

        write(
            hash_file_path,
            generate_sha256_file_content(source_hash, target_file.to_string_lossy()),
        )
        .wrap_err("Failed to write hash file.")?;
        info!("Write success!");

        info!("Starting cleanup.");

        info!("Parsing files of target directory for dates.");
        let backup_files = dates_from_directory(&target_dir_path)?;

        info!("Determine which files to keep...");

        let parse_cli_keep_count = |count: i32| -> Result<Option<u32>> {
            if count >= 0 {
                Ok(Some(u32::try_from(count)?))
            } else {
                Ok(None)
            }
        };

        let backup_files_to_keep = identify_files_to_keep(
            &backup_files,
            parse_cli_keep_count(cli.keep_newest_count)?,
            parse_cli_keep_count(cli.keep_daily_count)?,
            parse_cli_keep_count(cli.keep_monthly_count)?,
            parse_cli_keep_count(cli.keep_yearly_count)?,
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
        let files_to_trash_paths = files_to_trash.into_iter().map(|file| file.path);

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

    Cli::command().print_help()?;

    Ok(())
}
