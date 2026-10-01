// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  recording_controls_may_raise, recording_controls_may_restore, recording_ui_may_hide,
  region_scene_owner, region_selector_capture_affinity, region_selector_is_interactive,
  region_selector_may_show, region_selector_restores_opacity, screenshot_region_may_restore,
  RegionSelectorCaptureAffinity,
};
use crate::osc::scene::RegionSceneOwner;

#[test]
fn a_region_gesture_does_not_raise_the_source_selector() {
  assert!(recording_controls_may_raise(true, false, false));
  assert!(!recording_controls_may_raise(true, true, false));
  assert!(!recording_controls_may_raise(false, false, false));
}

#[test]
fn borrowed_recording_controls_cannot_be_raised() {
  assert!(!recording_controls_may_raise(true, false, true));
}

#[test]
fn borrowed_recording_controls_restore_only_to_the_idle_visible_ui() {
  assert!(recording_controls_may_restore(true, true));
  assert!(!recording_controls_may_restore(false, true));
  assert!(!recording_controls_may_restore(true, false));
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
fn a_screenshot_session_presents_the_borrowed_region_window() {
  assert!(region_selector_restores_opacity(true));
  assert!(region_selector_restores_opacity(false));
}

#[test]
fn a_running_region_keeps_its_scene_when_the_recording_bar_hides() {
  assert_eq!(
    region_scene_owner(false, false, false, false),
    RegionSceneOwner::Normal
  );
  assert_eq!(
    region_scene_owner(false, false, false, true),
    RegionSceneOwner::DormantNormal
  );
}

#[test]
fn the_shutter_excludes_region_then_restores_the_global_capture_preference() {
  assert_eq!(
    region_selector_capture_affinity(0.0, false),
    RegionSelectorCaptureAffinity {
      other_windows: false,
      region_selector: false,
    }
  );
  assert_eq!(
    region_selector_capture_affinity(0.0, true),
    RegionSelectorCaptureAffinity {
      other_windows: true,
      region_selector: false,
    }
  );
  assert_eq!(
    region_selector_capture_affinity(1.0, true),
    RegionSelectorCaptureAffinity {
      other_windows: true,
      region_selector: true,
    }
  );
}

#[test]
fn a_screenshot_session_keeps_its_driver_window_alive_for_cleanup() {
  assert!(!recording_ui_may_hide(true));
  assert!(recording_ui_may_hide(false));
}

#[test]
fn a_saved_region_mode_restores_only_with_visible_recording_controls() {
  assert!(screenshot_region_may_restore(true, true));
  assert!(!screenshot_region_may_restore(true, false));
  assert!(!screenshot_region_may_restore(false, true));
}
