// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::Path, time::SystemTime};

use color_eyre::eyre::Result;

pub fn modified_date<P>(file_path: P) -> Result<SystemTime>
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();
    let metadata = path.metadata()?;
    Ok(metadata.modified()?)
}
