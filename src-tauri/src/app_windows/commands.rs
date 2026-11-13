// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The commands the frontend invokes on the app's windows: the recording
//! UI, the island, the region selector, tool panels and listboxes. Tauri
//! takes one handler for the whole app and each invoke is handled once, so
//! these are told apart by name and the rest go on to the app's own list.

use tauri::ipc::Invoke;

const COMMANDS: [&str; 30] = [
  "window_ready",
  "show_inactive_cursor",
  "close_color_panel",
  "show_color_panel",
  "begin_region_selector_gesture",
  "finish_region_selector_gesture",
  "hide_recording_ui",
  "open_recording_ui",
  "display_notches",
  "finish_hiding_island",
  "island_presence",
  "island_ready_to_move",
  "set_island_hit_area",
  "recording_ui_visible",
  "toggle_recording_ui",
  "hide_region_selector",
  "hide_standalone_listbox",
  "open_standalone_listboxes",
  "set_recording_controls_borrowed",
  "begin_screenshot_drawing",
  "set_region_selector_opacity",
  "set_region_selector_passthrough",
  "set_screenshot_region_session",
  "show_region_selector",
  "prepare_screenshot_region_magnifier",
  "set_screenshot_region_osc",
  "set_region_selector_osc_frame_visible",
  "show_standalone_listbox",
  "move_standalone_listbox",
  "set_standalone_listbox_hit_area",
];

/// `app_commands` with the windows' commands beside them.
pub(crate) fn with_commands(
  app_commands: impl Fn(Invoke) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  let window_commands = window_commands();
  move |invoke| {
    if COMMANDS.contains(&invoke.message.command()) {
      window_commands(invoke)
    } else {
      app_commands(invoke)
    }
  }
}

fn window_commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  use super::*;
  tauri::generate_handler![
    on_demand::window_ready,
    inactive_cursor::show_inactive_cursor,
    color_panel::close_color_panel,
    color_panel::show_color_panel,
    region_gesture::begin_region_selector_gesture,
    region_gesture::finish_region_selector_gesture,
    hide_recording_ui,
    open_recording_ui,
    display_notch::display_notches,
    finish_hiding_island,
    island_presence,
    island_ready_to_move,
    set_island_hit_area,
    recording_ui_visible,
    toggle_recording_ui,
    region::hide_region_selector,
    options::hide_standalone_listbox,
    options::open_standalone_listboxes,
    region::set_recording_controls_borrowed,
    region::begin_screenshot_drawing,
    region::set_region_selector_opacity,
    region::set_region_selector_passthrough,
    region::set_screenshot_region_session,
    region::show_region_selector,
    screenshot_region::magnifier::prepare_screenshot_region_magnifier,
    screenshot_region::osc_command::set_screenshot_region_osc,
    screenshot_region::presentation::set_region_selector_osc_frame_visible,
    options::show_standalone_listbox,
    options::placement::move_standalone_listbox,
    options::hit_area::set_standalone_listbox_hit_area
  ]
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use super::COMMANDS;

  #[test]
  fn every_window_command_is_named_once() {
    assert_eq!(
      COMMANDS.iter().collect::<HashSet<_>>().len(),
      COMMANDS.len()
    );
  }
}
