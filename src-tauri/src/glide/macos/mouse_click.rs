// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn handle_mouse_down(
  app: &AppHandle,
  state: &SharedState,
  event: &CGEvent,
) -> CallbackResult {
  let settings = native_settings::snapshot();
  // The double tap and this click are one action under two inputs, so the one
  // setting turns both of them off.
  if crate::capture_overlays::blocks_glide(app)
    || crate::shortcuts::is_capturing()
    || !settings.enabled
    || !settings.double_tap_center
  {
    return CallbackResult::Keep;
  }
  if !native_settings::is_down(settings.mouse_modifier)
    || event.get_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE) != DOUBLE_CLICK_STATE
  {
    return CallbackResult::Keep;
  }
  let point = event.location();
  if let Some((target, frame, work_position, work_size)) = mouse_center_context(state) {
    end_session(app, state, true);
    set_mouse_up_swallow(state, true);
    center_captured(&target, frame, work_position, work_size);
    return CallbackResult::Drop;
  }
  if crate::glide::core::activity::BusyLease::is_busy() {
    return CallbackResult::Keep;
  }
  // One of ours answers natively, a foreign one through Accessibility; either
  // titlebar is a titlebar as far as this click is concerned.
  if !any_titlebar(app, point) {
    return CallbackResult::Keep;
  }
  if is_active(state)
    || native_settings::is_down(settings.monitors_modifier)
    || native_settings::is_down(settings.spaces_modifier)
  {
    return CallbackResult::Keep;
  }
  // An internal takeover, not a commit: minimizing the window this click is
  // about to center would be two destinations at once. The cancel's restore, if
  // the click landed on a glide that had already moved something, is preempted
  // by the centering below before the tween thread can take a single step.
  end_session(app, state, true);
  set_mouse_up_swallow(state, true);
  center_window_at(app, point);
  CallbackResult::Drop
}
