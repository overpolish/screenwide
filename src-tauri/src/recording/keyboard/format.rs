// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub(crate) const FORMAT_VERSION: u16 = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyboardModifier {
  Command,
  Control,
  Function,
  Option,
  Shift,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub enum KeyboardRecord {
  Header {
    platform: String,
    timebase: String,
    version: u16,
  },
  Shortcut {
    key_code: u16,
    modifiers: Vec<KeyboardModifier>,
    timestamp_us: u64,
  },
  KeyDown {
    key_code: u16,
    modifiers: Vec<KeyboardModifier>,
    timestamp_us: u64,
  },
  KeyUp {
    key_code: u16,
    modifiers: Vec<KeyboardModifier>,
    timestamp_us: u64,
  },
}

/// Reads every complete shortcut record. A crash may truncate only the final
/// JSON line; complete events before that line remain usable, like cursor data.
pub fn read(path: &Path) -> Result<Vec<KeyboardRecord>, String> {
  let reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
  let mut records = Vec::new();
  let mut parse_failure = None;
  for line in reader.lines() {
    let line = line.map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
      continue;
    }
    match serde_json::from_str(&line) {
      Ok(record) => records.push(record),
      Err(error) => {
        parse_failure = Some(error.to_string());
        break;
      }
    }
  }
  match records.first() {
    Some(KeyboardRecord::Header { version, .. }) if *version == 1 || *version == FORMAT_VERSION => {
      Ok(records)
    }
    Some(KeyboardRecord::Header { version, .. }) => Err(format!(
      "Keyboard shortcut recording version {version} is not supported"
    )),
    _ => Err(parse_failure.map_or_else(
      || "The keyboard shortcut recording has no valid header".to_owned(),
      |error| format!("The keyboard shortcut recording has no valid header: {error}"),
    )),
  }
}
