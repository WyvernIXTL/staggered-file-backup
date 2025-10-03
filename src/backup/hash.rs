// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsStr, fs::File, io};

use color_eyre::eyre::{Context, Result};
use sha2::{Digest, Sha256};

pub fn hash_file(file: &mut File) -> Result<String> {
    let mut hasher = Sha256::new();

    io::copy(file, &mut hasher).wrap_err("Failed to hash file.")?;

    let hash = hasher.finalize();

    Ok(hex::encode_upper(hash))
}

pub fn generate_sha256_file_content<S, S2>(hash: S, file_name: S2) -> String
where
    S: AsRef<str>,
    S2: AsRef<OsStr>,
{
    format!("{} *{}\n", hash.as_ref(), file_name.as_ref().display())
}
