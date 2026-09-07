// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use super::input_kind::InputKind;

static CANCELLED: AtomicBool = AtomicBool::new(false);

pub(super) fn owns(active: Option<InputKind>, incoming: InputKind) -> bool {
  active.is_none() || active == Some(incoming)
}

pub(super) fn wheel_delta(horizontal: bool, delta: i16) -> (f64, f64) {
  let step = f64::from(delta) / 120.0 * 36.0;
  if horizontal {
    (step, 0.0)
  } else {
    (-step, 0.0)
  }
}

pub(super) fn cancel_until_release() {
  crate::glide::core::trace::input("windows-navigation", "cancel-latched");
  CANCELLED.store(true, Ordering::Release);
}
pub(super) fn clear_cancel() {
  CANCELLED.store(false, Ordering::Release);
}
pub(super) fn cancelled() -> bool {
  CANCELLED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn active_input_owns_stream() {
    assert!(!owns(Some(InputKind::Mouse), InputKind::TrackpadScroll));
    assert!(owns(Some(InputKind::Mouse), InputKind::Mouse));
  }
  #[test]
  fn wheel_is_directional_and_scaled() {
    assert_eq!(wheel_delta(false, 120), (-36.0, 0.0));
    assert_eq!(wheel_delta(true, 120), (36.0, 0.0));
  }
  #[test]
  fn cancel_latches_until_explicit_clear() {
    clear_cancel();
    assert!(!cancelled());
    cancel_until_release();
    assert!(cancelled());
    clear_cancel();
    assert!(!cancelled());
  }
}
