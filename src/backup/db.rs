// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::Path;

use color_eyre::{
    Section,
    eyre::{Context, ContextCompat, Result, eyre},
};
use diesel::{Connection, SqliteConnection, sqlite::Sqlite};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const DB_NAME: &'static str = "staggered-file-backup.keepme";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn connect_db(backup_dir: impl AsRef<Path>) -> Result<SqliteConnection> {
    SqliteConnection::establish(
        backup_dir
            .as_ref()
            .join(DB_NAME)
            .to_str()
            .wrap_err("Backup tracking database expects a utf-8 compatible path.")
            .suggestion("Check if your backup directory path entails non utf-8 characters.")?,
    )
    .wrap_err("Failed to connect to backup tracking database located in backup folder.")
}

fn run_pending_migrations(conn: &mut impl MigrationHarness<Sqlite>) -> Result<()> {
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|err| eyre!(err))
        .wrap_err("Failed to run database migrations.")?;
    Ok(())
}
