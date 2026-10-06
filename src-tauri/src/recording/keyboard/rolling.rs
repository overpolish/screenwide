// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a replay clip of the keyboard opens: with the keys that were already
//! held when it begins, so a shortcut finished inside the clip still shows
//! its modifiers.

use super::KeyboardRecord;
use crate::recording::sidecar_output::{Baseline, TimedRecord};

impl TimedRecord for KeyboardRecord {
  fn timestamp_us(&self) -> Option<u64> {
    match self {
      Self::Header { .. } => None,
      Self::Shortcut { timestamp_us, .. }
      | Self::KeyDown { timestamp_us, .. }
      | Self::KeyUp { timestamp_us, .. }
      | Self::Typing { timestamp_us } => Some(*timestamp_us),
    }
  }

  fn at_timestamp_us(&self, at: u64) -> Self {
    let mut record = self.clone();
    match &mut record {
      Self::Header { .. } => {}
      Self::Shortcut { timestamp_us, .. }
      | Self::KeyDown { timestamp_us, .. }
      | Self::KeyUp { timestamp_us, .. }
      | Self::Typing { timestamp_us } => *timestamp_us = at,
    }
    record
  }
}

#[derive(Clone, Default)]
pub(crate) struct KeyboardBaseline {
  held: Vec<KeyboardRecord>,
}

impl Baseline<KeyboardRecord> for KeyboardBaseline {
  fn absorb(&mut self, record: &KeyboardRecord) {
    match record {
      KeyboardRecord::KeyDown { key_code, .. } => {
        let key_code = *key_code;
        self.held.retain(|held| !holds(held, key_code));
        self.held.push(record.clone());
      }
      KeyboardRecord::KeyUp { key_code, .. } => {
        let key_code = *key_code;
        self.held.retain(|held| !holds(held, key_code));
      }
      // Moments, not state.
      KeyboardRecord::Shortcut { .. }
      | KeyboardRecord::Typing { .. }
      | KeyboardRecord::Header { .. } => {}
    }
  }

  fn restate(&self) -> Vec<KeyboardRecord> {
    self.held.clone()
  }
}

fn holds(record: &KeyboardRecord, key: u16) -> bool {
  matches!(record, KeyboardRecord::KeyDown { key_code, .. } if *key_code == key)
}
