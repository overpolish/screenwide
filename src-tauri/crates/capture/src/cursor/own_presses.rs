// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Presses on Screenwide's own windows, left out of a recording that leaves
//! those windows out of its picture: a click on the recording dock lands on
//! nothing the recording shows, so no click effect or auto zoom may follow
//! it. A recording that shows those windows keeps them like any other.

use std::sync::{Arc, Mutex};

use super::{ButtonState, CursorButton, EventSink, RawCursorEventKind};

/// The buttons pressed on one of Screenwide's own windows and not yet let
/// go, so that their release is left out with them.
#[derive(Default)]
struct HeldOnOwn(Vec<CursorButton>);

impl HeldOnOwn {
  /// Whether a press or release is kept. Only a press asks `on_own_window`,
  /// because a release belongs wherever its press landed.
  fn keeps(
    &mut self,
    button: CursorButton,
    state: ButtonState,
    on_own_window: impl FnOnce() -> bool,
  ) -> bool {
    match state {
      ButtonState::Down => {
        if !on_own_window() {
          return true;
        }
        if !self.0.contains(&button) {
          self.0.push(button);
        }
        false
      }
      ButtonState::Up => match self.0.iter().position(|&held| held == button) {
        Some(index) => {
          self.0.swap_remove(index);
          false
        }
        None => true,
      },
    }
  }
}

/// `sink`, passing on no press that lands on one of Screenwide's own windows
/// and no release of one. `on_own_window` takes a global pointer position in
/// the recorder's own coordinates.
pub(super) fn without_own_presses(
  sink: EventSink,
  on_own_window: fn(f64, f64) -> bool,
) -> EventSink {
  let held = Mutex::new(HeldOnOwn::default());
  Arc::new(move |event| {
    if let RawCursorEventKind::Button { button, state, .. } = event.kind {
      let keeps = held
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .keeps(button, state, || on_own_window(event.x, event.y));
      if !keeps {
        return false;
      }
    }
    sink(event)
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_press_on_screenwide_is_left_out_with_its_release_and_nothing_else() {
    let mut held = HeldOnOwn::default();
    assert!(!held.keeps(CursorButton::Left, ButtonState::Down, || true));
    // Another button pressed and let go on the work meanwhile is kept.
    assert!(held.keeps(CursorButton::Right, ButtonState::Down, || false));
    assert!(held.keeps(CursorButton::Right, ButtonState::Up, || true));
    assert!(!held.keeps(CursorButton::Left, ButtonState::Up, || false));
    // The next press on the work, and its release, are kept.
    assert!(held.keeps(CursorButton::Left, ButtonState::Down, || false));
    assert!(held.keeps(CursorButton::Left, ButtonState::Up, || false));
  }
}
