// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{PlaybackMode, RecordingPreviewPlayerState};
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_timeline_edit::DeletedKeyboardShortcutRange;
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
    .map_err(|_| "The keyboard preview settings are unavailable".to_owned())? = keyboard_effects;
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
  playback_ranges: Vec<super::RecordingPreviewPlaybackRange>,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let sources = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?;
  let keyboard = sources.keyboard.clone();
  *sources
    .animation_ranges
    .write()
    .map_err(|_| "The animation timeline is unavailable".to_owned())? =
    super::animation_timeline_ranges(&playback_ranges);
  if let Some(keyboard) = keyboard {
    keyboard.set_deleted_shortcuts(&shortcut_ids, &shortcut_ranges);
  }
  super::audio_visualizer::refresh_audio_ribbon(&manager);
  if !manager.is_playing {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}
