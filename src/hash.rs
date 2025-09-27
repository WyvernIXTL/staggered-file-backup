// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    fs::OpenOptions,
    io::{BufReader, Read},
    path::Path,
};

use color_eyre::eyre::Result;

pub fn hash_file<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let file = OpenOptions::new().read(true).open(path.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut hasher = hmac_sha256::Hash::new();

    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode_upper(hash))
}

pub fn generate_sha256_file_content<S, S2>(hash: S2, file_name: S) -> String
where
    S: AsRef<str>,
    S2: AsRef<str>,
{
    format!("{} *{}\n", hash.as_ref(), file_name.as_ref())
}
