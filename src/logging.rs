// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fs::{OpenOptions, create_dir_all};

use clap::CommandFactory;
use color_eyre::eyre::{Result, eyre};
use log::{LevelFilter, info};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

use crate::Cli;

pub fn setup_logging() -> Result<()> {
    let dirs = directories::BaseDirs::new()
        .ok_or(eyre!("Failed getting base dirs like AppData on Windows."))?;

    let binding = Cli::command();
    let command_name = binding.get_name();

    let data_dir = dirs.data_dir();
    let app_dir = data_dir.join(command_name);
    create_dir_all(&app_dir)?;
    let log_file = app_dir.join(format!("{}.log", command_name));

    let log_file_handle = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(&log_file)?;

    let _ = CombinedLogger::init(vec![
        (TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )),
        (WriteLogger::new(LevelFilter::Info, Config::default(), log_file_handle)),
    ]);

    info!("Logs are written to: '{}'", log_file.display());

    Ok(())
}
