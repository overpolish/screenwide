// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Parses physical key transitions without assigning display lifetimes.

use serde_json::Value;
use std::{
  fs::File,
  io::{BufRead, BufReader},
  path::Path,
};

use super::{KeyPress, Shortcut};

pub(super) fn read_values(path: &Path) -> Result<Vec<Value>, String> {
  let reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
  reader
    .lines()
    .map_while(Result::ok)
    .filter(|line| !line.trim().is_empty())
    .map(|line| serde_json::from_str(&line).map_err(|error| error.to_string()))
    .collect()
}

pub(super) fn parse_v1(records: &[Value]) -> Vec<Shortcut> {
  records
    .iter()
    .filter(|record| {
      record
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.eq_ignore_ascii_case("shortcut"))
    })
    .filter_map(|record| {
      let timestamp = record.get("timestampUs")?.as_u64()?;
      Some(Shortcut {
        keys: vec![KeyPress {
          key_code: record.get("keyCode")?.as_u64()? as u16,
          modifier_mask: modifier_mask(record.get("modifiers")?),
          down_us: timestamp,
          // Version-one sidecars stored one completed shortcut timestamp.
          up_us: Some(timestamp),
        }],
      })
    })
    .collect()
}

pub(super) fn reconstruct_v2(records: &[Value]) -> Vec<Shortcut> {
  let mut builder = Builder::default();
  for record in records
    .iter()
    .filter(|record| record.get("type").and_then(Value::as_str) != Some("header"))
  {
    let kind = record
      .get("type")
      .and_then(Value::as_str)
      .unwrap_or_default();
    let key = record
      .get("keyCode")
      .and_then(Value::as_u64)
      .unwrap_or_default() as u16;
    let at = record
      .get("timestampUs")
      .and_then(Value::as_u64)
      .unwrap_or_default();
    if kind.eq_ignore_ascii_case("keyDown") || kind.eq_ignore_ascii_case("down") {
      builder.press(
        key,
        at,
        modifier_mask(record.get("modifiers").unwrap_or(&Value::Null)),
      );
    } else if kind.eq_ignore_ascii_case("keyUp") || kind.eq_ignore_ascii_case("up") {
      builder.release(key, at);
    }
  }
  builder.shortcuts
}

#[derive(Default)]
struct Builder {
  active: Vec<(u16, usize)>,
  shortcuts: Vec<Shortcut>,
}

impl Builder {
  fn press(&mut self, key: u16, at: u64, modifier_mask: u32) {
    if self.active.iter().any(|(active, _)| *active == key) {
      return;
    }
    if self.active.is_empty() {
      self.shortcuts.push(Shortcut { keys: Vec::new() });
    }
    let Some(shortcut) = self.shortcuts.last_mut() else {
      return;
    };
    shortcut.keys.push(KeyPress {
      key_code: key,
      modifier_mask,
      down_us: at,
      up_us: None,
    });
    self.active.push((key, shortcut.keys.len() - 1));
  }

  fn release(&mut self, key: u16, at: u64) {
    let Some(position) = self.active.iter().position(|(active, _)| *active == key) else {
      return;
    };
    let (_, index) = self.active.remove(position);
    let Some(shortcut) = self.shortcuts.last_mut() else {
      return;
    };
    if let Some(press) = shortcut.keys.get_mut(index) {
      press.up_us = Some(at);
    }
  }
}

fn modifier_mask(value: &Value) -> u32 {
  value
    .as_array()
    .into_iter()
    .flatten()
    .filter_map(Value::as_str)
    .fold(0, |mask, modifier| {
      mask
        | match modifier {
          "command" => 1,
          "control" => 2,
          "option" => 4,
          "shift" => 8,
          "function" => 16,
          _ => 0,
        }
    })
}
