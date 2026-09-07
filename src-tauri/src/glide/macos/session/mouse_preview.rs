// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Immediate preview for the reserved mouse control.

use tauri::AppHandle;

use super::{active_input, InputKind, SharedState};
use crate::glide::platform::native_settings;

pub(in crate::glide::platform) fn poll(app: &AppHandle, state: &SharedState) {
  let settings = native_settings::snapshot();
  let mouse_down = native_settings::is_down(settings.mouse_modifier);
  let blocked = !settings.enabled
    || crate::shortcuts::is_capturing()
    || crate::capture_overlays::blocks_glide(app);
  if !mouse_down {
    crate::glide::core::trace::input("mac-mouse", "mouse-control-released");
    super::set_suppression(state, InputKind::Mouse, false);
  }
  if active_input(state) == Some(InputKind::Mouse)
    && !super::monitor_mode(state)
    && (!mouse_down || blocked)
  {
    crate::glide::core::trace::input("mac-mouse", "end-normal-mouse-preview");
    super::end_session(app, state, blocked);
  }
}
