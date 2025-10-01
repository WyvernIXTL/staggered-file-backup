// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser, ValueEnum, ValueHint};
use clap_complete::Shell;
use color_eyre::eyre::{Ok, Result};
use license_fetcher::read_package_list_from_out_dir;

use crate::{logging::setup_logging, setup::setup_hooks};

mod backup;
mod logging;
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
        let parse_cli_keep_count = |count: i32| -> Result<Option<u32>> {
            if count >= 0 {
                Ok(Some(u32::try_from(count)?))
            } else {
                Ok(None)
            }
        };

        return backup::backup(
            source_path,
            target_dir_path,
            parse_cli_keep_count(cli.keep_newest_count)?,
            parse_cli_keep_count(cli.keep_daily_count)?,
            parse_cli_keep_count(cli.keep_monthly_count)?,
            parse_cli_keep_count(cli.keep_yearly_count)?,
        );
    }

    Cli::command().print_help()?;

    Ok(())
}
