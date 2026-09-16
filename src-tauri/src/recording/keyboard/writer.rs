// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Privacy filtering and JSONL writing for keyboard events.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};

use super::{
  FocusContext, KeyboardModifier, KeyboardRecord, RawKeyboardEvent, RawKeyboardEventKind,
};
use crate::recording::clock::SidecarClock;

pub(super) struct StreamWriter {
  pub(super) active_keys: HashSet<u16>,
  pub(super) clock: SidecarClock,
  pub(super) failure: Option<String>,
  pub(super) writer: BufWriter<File>,
}

pub(super) fn modifier_transition_is_down(was_active: bool, aggregate_flag: bool) -> bool {
  aggregate_flag && !was_active
}

impl StreamWriter {
  pub(super) fn accepts(event: &RawKeyboardEvent) -> bool {
    let RawKeyboardEventKind::KeyDown {
      is_printable,
      is_repeat,
    } = event.kind
    else {
      return true;
    };
    if is_repeat || event.focus == FocusContext::Secure {
      return false;
    }
    if is_printable && event.focus != FocusContext::NonText {
      return event.modifiers.iter().any(|modifier| {
        matches!(
          modifier,
          KeyboardModifier::Command | KeyboardModifier::Control
        )
      });
    }
    true
  }

  pub(super) fn record(&mut self, event: RawKeyboardEvent) -> Result<bool, String> {
    let Some(timestamp_us) = self.clock.timestamp_us(event.at) else {
      return Ok(false);
    };
    let record = match event.kind {
      RawKeyboardEventKind::KeyDown { .. } => {
        if !Self::accepts(&event) || !self.active_keys.insert(event.key_code) {
          return Ok(false);
        }
        KeyboardRecord::KeyDown {
          key_code: event.key_code,
          modifiers: event.modifiers,
          timestamp_us,
        }
      }
      RawKeyboardEventKind::KeyUp => {
        if !self.active_keys.remove(&event.key_code) {
          return Ok(false);
        }
        KeyboardRecord::KeyUp {
          key_code: event.key_code,
          modifiers: event.modifiers,
          timestamp_us,
        }
      }
      RawKeyboardEventKind::FlagsChanged { is_down, modifier } => {
        // The event flag is aggregate across left/right variants. When both
        // Shift keys are held, releasing one leaves the aggregate flag set;
        // the tracked physical key wins in that case.
        let is_down =
          modifier_transition_is_down(self.active_keys.contains(&event.key_code), is_down);
        if is_down {
          if !self.active_keys.insert(event.key_code) {
            return Ok(false);
          }
          KeyboardRecord::KeyDown {
            key_code: event.key_code,
            modifiers: vec![modifier],
            timestamp_us,
          }
        } else {
          if !self.active_keys.remove(&event.key_code) {
            return Ok(false);
          }
          KeyboardRecord::KeyUp {
            key_code: event.key_code,
            modifiers: vec![modifier],
            timestamp_us,
          }
        }
      }
    };
    serde_json::to_writer(&mut self.writer, &record).map_err(|error| error.to_string())?;
    self
      .writer
      .write_all(b"\n")
      .map_err(|error| error.to_string())?;
    self.writer.flush().map_err(|error| error.to_string())?;
    Ok(true)
  }
}
