// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone's speech tools' commands, handed to Tauri beside the
//! app's (`crate::invoke_handler`), which takes one handler for the whole
//! app.

use tauri::ipc::Invoke;

use super::commands::switches::{self, ducking_switch, restore_voice_switch};
use super::{commands, marks, transcript};

const COMMANDS: [&str; 15] = [
  "plan_recording_silences",
  "get_recording_noise_reduction",
  "set_recording_noise_reduction",
  "get_recording_vocal_cleanup",
  "set_recording_vocal_cleanup",
  "get_recording_auto_volume",
  "set_recording_auto_volume",
  "get_recording_smart_ducking",
  "set_recording_smart_ducking",
  "get_recording_restore_voice",
  "set_recording_restore_voice",
  "transcribe_recording",
  "cancel_recording_transcription",
  "get_recording_transcript",
  "find_transcript_marks",
];

/// `app_commands` with the speech tools' commands beside them.
pub(crate) fn with_commands(
  app_commands: impl Fn(Invoke) -> bool + Send + Sync + 'static,
) -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  let speech_commands = speech_commands();
  move |invoke| {
    if COMMANDS.contains(&invoke.message.command()) {
      speech_commands(invoke)
    } else {
      app_commands(invoke)
    }
  }
}

fn speech_commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  tauri::generate_handler![
    commands::plan_recording_silences,
    switches::get_recording_noise_reduction,
    switches::set_recording_noise_reduction,
    switches::get_recording_vocal_cleanup,
    switches::set_recording_vocal_cleanup,
    switches::get_recording_auto_volume,
    switches::set_recording_auto_volume,
    ducking_switch::get_recording_smart_ducking,
    ducking_switch::set_recording_smart_ducking,
    restore_voice_switch::get_recording_restore_voice,
    restore_voice_switch::set_recording_restore_voice,
    transcript::transcribe_recording,
    transcript::cancel_recording_transcription,
    transcript::get_recording_transcript,
    marks::find_transcript_marks,
  ]
}
