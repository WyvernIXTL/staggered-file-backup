// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsString, path::PathBuf, str::FromStr};

use clap::{CommandFactory, Parser, ValueEnum, ValueHint};
use clap_complete::Shell;
use color_eyre::eyre::{ContextCompat, Ok, Result};
use file::modified_date_string_from_path;
use license_fetcher::read_package_list_from_out_dir;
use log::{info, warn};
use logging::setup_logging;

mod file;
mod logging;

fn parse_str_to_source_pathbuf(s: &str) -> std::result::Result<PathBuf, String> {
    match PathBuf::from_str(s) {
        std::result::Result::Ok(path_buf) => {
            if path_buf.is_file() && path_buf.exists() {
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
            if path_buf.is_dir() {
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
    color_eyre::install()?;

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
        setup_logging()?;

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
        let modified_string = modified_date_string_from_path(source_path)?;
        info!("Source file last modified: {}", &modified_string);

        info!("Target directory: {}", target_dir_path.display());

        let mut target_file = OsString::from(modified_string);
        target_file.push("_");
        target_file.push(source_basename);

        if let Some(ext) = extension_option {
            target_file.push(".");
            target_file.push(ext);
        }

        info!("Target file: {}", target_file.display());

        let target_file_path = target_dir_path.join(target_file);
        info!("Target file path: {}", target_file_path.display());

        return Ok(());
    }

    Cli::command().print_help()?;

    Ok(())
}
