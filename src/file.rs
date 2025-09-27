// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use color_eyre::{
    Section,
    eyre::{Context, Result},
};

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
