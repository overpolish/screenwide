// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Version 3 adds where the text caret was while typing.
pub const FORMAT_VERSION: u16 = 3;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyboardModifier {
  Command,
  Control,
  Function,
  Option,
  Shift,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
  /// Text was being typed. A key typed into a text field is never recorded,
  /// so this says only when, and no more often than the writer allows.
  Typing { timestamp_us: u64 },
  /// Where the text caret was while text was typed, in the coordinates the
  /// cursor file's header names: a box a line high and, for a caret, about a
  /// point wide. It says where the text goes on the screen, never what it
  /// is, and is never kept for a password field.
  Caret {
    timestamp_us: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
  },
}

/// Whether `key_code` is a modifier: Command, Shift, Caps Lock, Option,
/// Control or Function.
pub fn is_modifier_key(key_code: u16) -> bool {
  matches!(key_code, 54..=63)
}

/// Whether a key types rather than makes a shortcut: any key but a modifier,
/// pressed without Command or Control (`command_or_control`). Return,
/// Backspace, an arrow and a letter typed outside a text field all type.
pub fn types(key_code: u16, command_or_control: bool) -> bool {
  !is_modifier_key(key_code) && !command_or_control
}

/// When `records` show typing: the typing marks, and every key pressed that
/// [`types`].
pub fn typing_times_us(records: &[KeyboardRecord]) -> Vec<u64> {
  let mut times: Vec<u64> = records
    .iter()
    .filter_map(|record| match record {
      KeyboardRecord::Typing { timestamp_us } => Some(*timestamp_us),
      KeyboardRecord::KeyDown {
        key_code,
        modifiers,
        timestamp_us,
      } if types(
        *key_code,
        modifiers.iter().any(|modifier| {
          matches!(
            modifier,
            KeyboardModifier::Command | KeyboardModifier::Control
          )
        }),
      ) =>
      {
        Some(*timestamp_us)
      }
      _ => None,
    })
    .collect();
  times.sort_unstable();
  times
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
    Some(KeyboardRecord::Header { version, .. }) if (1..=FORMAT_VERSION).contains(version) => {
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
