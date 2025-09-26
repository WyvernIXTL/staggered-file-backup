// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use clap::{CommandFactory, Parser, ValueEnum, ValueHint};
use clap_complete::Shell;
use color_eyre::eyre::{Ok, Result};
use license_fetcher::read_package_list_from_out_dir;
use logging::setup_logging;

mod logging;

/// An easy and secure staggered file backup solution
#[derive(Parser, Debug)]
#[command(version, about, author)]
struct Cli {
    /// Path to file to be backed up
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    source: Option<PathBuf>,

    /// Path to folder to place backups in
    ///
    /// Please do not use the folder for anything else!
    #[arg(value_name = "TARGET_FOLDER", value_hint = ValueHint::DirPath)]
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

    setup_logging()?;

    log::info!("Hello World!");

    Ok(())
}
