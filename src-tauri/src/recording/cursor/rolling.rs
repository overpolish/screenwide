// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a replay clip of the cursor opens: in the shape, place, visibility and
//! pressed buttons the pointer already had when the clip begins.

use super::format::{ButtonState, CursorButton, CursorRecord};
use crate::recording::sidecar_output::{Baseline, TimedRecord};

impl TimedRecord for CursorRecord {
  fn timestamp_us(&self) -> Option<u64> {
    match self {
      Self::Header { .. } => None,
      Self::Appearance { timestamp_us, .. }
      | Self::Visibility { timestamp_us, .. }
      | Self::Position { timestamp_us, .. }
      | Self::Button { timestamp_us, .. }
      | Self::AppSwitch { timestamp_us } => Some(*timestamp_us),
    }
  }

  fn at_timestamp_us(&self, at: u64) -> Self {
    let mut record = self.clone();
    match &mut record {
      Self::Header { .. } => {}
      Self::Appearance { timestamp_us, .. }
      | Self::Visibility { timestamp_us, .. }
      | Self::Position { timestamp_us, .. }
      | Self::Button { timestamp_us, .. }
      | Self::AppSwitch { timestamp_us } => *timestamp_us = at,
    }
    record
  }
}

#[derive(Clone, Default)]
pub(crate) struct CursorBaseline {
  appearance: Option<CursorRecord>,
  position: Option<CursorRecord>,
  pressed: Vec<CursorRecord>,
  visibility: Option<CursorRecord>,
}

fn button_of(record: &CursorRecord) -> Option<CursorButton> {
  match record {
    CursorRecord::Button { button, .. } => Some(*button),
    _ => None,
  }
}

impl Baseline<CursorRecord> for CursorBaseline {
  fn absorb(&mut self, record: &CursorRecord) {
    match record {
      CursorRecord::Appearance { .. } => self.appearance = Some(record.clone()),
      CursorRecord::Position { .. } => self.position = Some(record.clone()),
      CursorRecord::Visibility { .. } => self.visibility = Some(record.clone()),
      CursorRecord::Button { button, state, .. } => {
        self.pressed.retain(|held| button_of(held) != Some(*button));
        if *state == ButtonState::Down {
          self.pressed.push(record.clone());
        }
      }
      // A moment, not a state: an app that came forward before the clip says
      // nothing about the clip.
      CursorRecord::AppSwitch { .. } | CursorRecord::Header { .. } => {}
    }
  }

  fn restate(&self) -> Vec<CursorRecord> {
    // A shown cursor is only ever written as a return from hiding, so a
    // visible state needs no record of its own.
    let hidden = self
      .visibility
      .as_ref()
      .filter(|record| matches!(record, CursorRecord::Visibility { visible: false, .. }));
    self
      .appearance
      .iter()
      .chain(&self.position)
      .chain(hidden)
      .chain(&self.pressed)
      .cloned()
      .collect()
  }
}
