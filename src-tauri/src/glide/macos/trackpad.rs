// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps a mouse-controlled trackpad episode out of Glide from first contact
//! through lift, even if its control is released partway through.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use super::{
  multitouch,
  session::{active_input, end_session, InputKind, SharedState},
};

const PHASE_ENDED: i64 = 4;
const PHASE_CANCELLED: i64 = 8;
static IGNORING: AtomicBool = AtomicBool::new(false);

pub(super) fn ignore_episode(phase: i64, mouse_modifier_down: bool) -> bool {
  if mouse_modifier_down {
    IGNORING.store(true, Ordering::Relaxed);
  }
  let ignoring = IGNORING.load(Ordering::Relaxed);
  if ignoring && phase & (PHASE_ENDED | PHASE_CANCELLED) != 0 {
    IGNORING.store(false, Ordering::Relaxed);
  }
  ignoring
}

pub(super) fn ignore_pointer(
  app: &AppHandle,
  state: &SharedState,
  mouse_modifier_down: bool,
) -> bool {
  if !mouse_modifier_down || !multitouch::pointer_episode_active() {
    return false;
  }
  if active_input(state) == Some(InputKind::Mouse) {
    end_session(app, state, true);
  }
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mouse_control_ignores_the_complete_contact_episode() {
    assert!(ignore_episode(1, true));
    assert!(ignore_episode(2, false));
    assert!(ignore_episode(PHASE_ENDED, false));
    assert!(!ignore_episode(1, false));
  }
}
