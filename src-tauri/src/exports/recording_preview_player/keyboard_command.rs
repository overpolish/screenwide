// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{PlaybackMode, RecordingPreviewPlayerState};
use crate::exports::keyboard_effects::KeyboardEffectSettings;
use crate::exports::timeline_edit::{DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange};
use tauri::State;

#[tauri::command]
pub(crate) fn set_recording_preview_keyboard_effects(
  state: State<'_, RecordingPreviewPlayerState>,
  keyboard_effects: KeyboardEffectSettings,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let settings = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?
    .keyboard_settings
    .clone();
  *settings
    .write()
    .map_err(|_| "The keyboard preview settings are unavailable".to_owned())? =
    keyboard_effects.normalized();
  if !manager.is_playing {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}

#[tauri::command]
pub(crate) fn set_recording_preview_deleted_keyboard_shortcuts(
  state: State<'_, RecordingPreviewPlayerState>,
  shortcut_ids: Vec<u64>,
  shortcut_ranges: Vec<DeletedKeyboardShortcutRange>,
  shortcut_positions: Vec<KeyboardShortcutPositionRange>,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let keyboard = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?
    .keyboard
    .clone();
  if let Some(keyboard) = keyboard {
    keyboard.set_deleted_shortcuts(&shortcut_ids, &shortcut_ranges);
    keyboard.set_shortcut_positions(&shortcut_positions);
  }
  if !manager.is_playing {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}
