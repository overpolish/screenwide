// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  recording_controls_may_raise, recording_ui_may_hide, region_selector_is_interactive,
  region_selector_may_show, region_selector_restores_opacity,
  source_selector_visibility_allows_show,
};

#[test]
fn a_region_gesture_does_not_raise_the_source_selector() {
  assert!(recording_controls_may_raise(true, false));
  assert!(!recording_controls_may_raise(true, true));
  assert!(!recording_controls_may_raise(false, false));
}

#[test]
fn a_region_gesture_keeps_the_source_selector_hidden_during_store_sync() {
  assert!(source_selector_visibility_allows_show(true, false));
  assert!(!source_selector_visibility_allows_show(true, true));
  assert!(!source_selector_visibility_allows_show(false, false));
}

#[test]
fn the_region_overlay_takes_clicks_only_while_interactive_outside_a_recording() {
  assert!(region_selector_is_interactive(true, true));
  assert!(!region_selector_is_interactive(false, true));
  assert!(!region_selector_is_interactive(true, false));
  assert!(!region_selector_is_interactive(false, false));
}

#[test]
fn a_hidden_recording_ui_cannot_resurrect_only_its_region_overlay() {
  assert!(region_selector_may_show(true, true, false));
  assert!(region_selector_may_show(false, false, false));
  assert!(!region_selector_may_show(true, false, false));
}

#[test]
fn a_screenshot_session_shows_the_overlay_without_the_recording_ui() {
  assert!(region_selector_may_show(true, false, true));
  assert!(region_selector_may_show(true, true, true));
}

#[test]
fn a_screenshot_session_preserves_prepared_window_opacity() {
  assert!(!region_selector_restores_opacity(true));
  assert!(region_selector_restores_opacity(false));
}

#[test]
fn a_screenshot_session_keeps_its_driver_window_alive_for_cleanup() {
  assert!(!recording_ui_may_hide(true));
  assert!(recording_ui_may_hide(false));
}
