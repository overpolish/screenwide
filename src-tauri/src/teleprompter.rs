// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The teleprompter, a paid tool. A script is a project of its own
//! (`scripts`), listed in the projects window and opened in a floating
//! window docked under the camera, which holds its controls, is where the
//! script is written, and no recording, screenshot or screen share shows
//! (`window`). It moves at a steady pace or follows the voice, heard with
//! dictation's model (`voice`), and shows the cues pinned to its words
//! (`cues`).

mod cue_window;
mod cues;
mod scripts;
mod voice;
mod window;

#[cfg(target_os = "windows")]
pub(crate) use cue_window::LABEL as CUE_LABEL;
#[cfg(target_os = "windows")]
pub(crate) use window::LABEL;
pub(crate) use window::{close_on, open, renamed};

use tauri::ipc::Invoke;

const COMMANDS: [&str; 14] = [
  "new_script",
  "read_script",
  "save_script",
  "teleprompter_script",
  "set_teleprompter_fullscreen",
  "start_teleprompter_voice",
  "stop_teleprompter_voice",
  "read_cues",
  "save_cues",
  "cue_preview",
  "add_cue",
  "show_teleprompter_cue",
  "teleprompter_cue",
  "hide_teleprompter_cue",
];

/// `app_commands` with the teleprompter's commands beside them.
pub(crate) fn with_commands(
  app_commands: impl Fn(Invoke) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  let teleprompter_commands: fn(Invoke) -> bool = tauri::generate_handler![
    scripts::new_script,
    scripts::read_script,
    scripts::save_script,
    window::teleprompter_script,
    window::set_teleprompter_fullscreen,
    voice::start_teleprompter_voice,
    voice::stop_teleprompter_voice,
    cues::read_cues,
    cues::save_cues,
    cues::cue_preview,
    cues::add_cue,
    cue_window::show_teleprompter_cue,
    cue_window::teleprompter_cue,
    cue_window::hide_teleprompter_cue,
  ];
  move |invoke| {
    if COMMANDS.contains(&invoke.message.command()) {
      teleprompter_commands(invoke)
    } else {
      app_commands(invoke)
    }
  }
}
