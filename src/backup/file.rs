// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ffi::{OsStr, OsString};

use color_eyre::eyre::{Ok, Result};

pub fn target_file_name(
    uuid: impl AsRef<str>,
    base_name: impl AsRef<OsStr>,
    extension: Option<impl AsRef<OsStr>>,
) -> Result<OsString> {
    let mut file_name = OsString::new();
    file_name.push(uuid.as_ref());
    file_name.push(base_name.as_ref());

    if let Some(ext) = extension.as_ref() {
        file_name.push(".");
        file_name.push(ext.as_ref());
    }

    Ok(file_name)
}
