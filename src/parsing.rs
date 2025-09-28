// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::OsStr, sync::LazyLock};

use log::error;
use regex::Regex;

fn parse_file_name(file_name: impl AsRef<OsStr>) -> Option<(u32, u32, u32)> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?<year>\d{4})\-(?<month>\d{2})\-(?<day>\d{2})\_.*$")
            .expect("Failed parsing regex")
    });

    let file_name_string = file_name.as_ref().to_string_lossy();

    let capture = REGEX.captures(&file_name_string)?;

    let year_str = capture.name("year")?.as_str();
    let month_str = capture.name("month")?.as_str();
    let day_str = capture.name("day")?.as_str();

    let year: u32 = year_str.parse().inspect_err(|err| error!("{}", err)).ok()?;
    let month: u32 = month_str
        .parse()
        .inspect_err(|err| error!("{}", err))
        .ok()?;
    let day: u32 = day_str.parse().inspect_err(|err| error!("{}", err)).ok()?;

    Some((year, month, day))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_file_name_valid() {
        let file_name = "2025-09-27_file1.txt.sha256";

        let result = parse_file_name(file_name);

        assert_eq!(result, Some((2025, 9, 27)))
    }

    #[test]
    fn test_parse_file_name_invalid() {
        let file_name = "23-09-27_file1.txt.sha256";

        let result = parse_file_name(file_name);

        assert_eq!(result, None)
    }
}
