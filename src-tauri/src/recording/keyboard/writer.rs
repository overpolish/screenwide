// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Privacy filtering and JSONL writing for keyboard events.

use std::collections::HashSet;

use super::rolling::KeyboardBaseline;
use super::{
  FocusContext, KeyboardModifier, KeyboardRecord, RawKeyboardEvent, RawKeyboardEventKind,
};
use crate::recording::clock::SidecarClock;
use crate::recording::sidecar_output::SidecarOutput;

/// The least time between two typing marks. Typing is what hides the cursor
/// and holds a zoom on, both of which need only roughly when it happened, so
/// the pace of the keys is not kept.
const TYPING_MARK_US: u64 = 500_000;

pub(super) struct StreamWriter {
  pub(super) active_keys: HashSet<u16>,
  pub(super) clock: SidecarClock,
  pub(super) failure: Option<String>,
  pub(super) output: SidecarOutput<KeyboardRecord, KeyboardBaseline>,
  /// When the last typing mark was written.
  pub(super) last_typing_us: Option<u64>,
  /// Modifier presses not written yet. Until the key they go with arrives
  /// they may belong to a hidden shortcut, which takes them away with it.
  pending_modifiers: Vec<KeyboardRecord>,
}

pub(super) fn modifier_transition_is_down(was_active: bool, aggregate_flag: bool) -> bool {
  aggregate_flag && !was_active
}

impl StreamWriter {
  pub(super) fn new(
    clock: SidecarClock,
    output: SidecarOutput<KeyboardRecord, KeyboardBaseline>,
  ) -> Self {
    Self {
      active_keys: HashSet::new(),
      clock,
      failure: None,
      output,
      last_typing_us: None,
      pending_modifiers: Vec::new(),
    }
  }

  /// Writes the modifier presses held back so far, in the order they came.
  pub(super) fn write_pending(&mut self) -> Result<(), String> {
    for record in std::mem::take(&mut self.pending_modifiers) {
      self.output.write(&record)?;
    }
    Ok(())
  }

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

  /// Whether `event` is a key typed into a text field: one `accepts` keeps
  /// out for what it would give away, and which is noted only as typing. A
  /// password field gives nothing away, not even that.
  fn types(event: &RawKeyboardEvent) -> bool {
    matches!(
      event.kind,
      RawKeyboardEventKind::KeyDown {
        is_printable: true,
        is_repeat: false,
      }
    ) && !matches!(event.focus, FocusContext::NonText | FocusContext::Secure)
      && !event.modifiers.iter().any(|modifier| {
        matches!(
          modifier,
          KeyboardModifier::Command | KeyboardModifier::Control
        )
      })
  }

  pub(super) fn record(&mut self, event: RawKeyboardEvent) -> Result<bool, String> {
    let Some(timestamp_us) = self.clock.timestamp_us(event.at) else {
      return Ok(false);
    };
    // A hidden shortcut leaves no trace: its key down is never taken in, so
    // its key up finds no accepted key, and the modifiers held back for it
    // are dropped and forgotten, so their releases are dropped too.
    if matches!(event.kind, RawKeyboardEventKind::KeyDown { .. })
      && super::hidden::is_hidden(event.key_code, &event.modifiers)
    {
      for record in std::mem::take(&mut self.pending_modifiers) {
        if let KeyboardRecord::KeyDown { key_code, .. } = record {
          self.active_keys.remove(&key_code);
        }
      }
      return Ok(false);
    }
    let record = match event.kind {
      RawKeyboardEventKind::KeyDown { .. } if Self::types(&event) => {
        if self
          .last_typing_us
          .is_some_and(|last| timestamp_us < last + TYPING_MARK_US)
        {
          return Ok(false);
        }
        self.last_typing_us = Some(timestamp_us);
        KeyboardRecord::Typing { timestamp_us }
      }
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
          self.pending_modifiers.push(KeyboardRecord::KeyDown {
            key_code: event.key_code,
            modifiers: vec![modifier],
            timestamp_us,
          });
          return Ok(true);
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
    self.write_pending()?;
    self.output.write(&record)?;
    self.output.flush()?;
    Ok(true)
  }
}
