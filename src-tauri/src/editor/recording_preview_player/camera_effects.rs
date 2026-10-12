// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as the preview of the open recording draws them:
//! changed, they are drawn over the frames the preview holds at once, and
//! the timeline is told to make its thumbnails again with them.

use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Emitter, Manager};

use super::{PlayerSources, RecordingPreviewPlayerState};
use crate::editor::camera_effects::live;
use screenwide_export::camera_effects::live::LiveCameraEffects;

/// Sent with the recording's id when its camera is drawn differently.
const CHANGED_EVENT: &str = "editor://camera-effects-changed";

/// The camera's effects as a player holds them, changed while it plays.
pub(super) type SharedEffects = Arc<RwLock<Option<Arc<LiveCameraEffects>>>>;

/// The effects of the recording open as `artifact_id`, as its project has
/// them now.
pub(super) fn shared(app: &AppHandle, artifact_id: u64) -> SharedEffects {
  Arc::new(RwLock::new(live::for_recording(app, artifact_id)))
}

impl PlayerSources {
  /// The camera's effects as they stand.
  pub(super) fn camera_effects_now(&self) -> Option<Arc<LiveCameraEffects>> {
    self
      .camera_effects
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }
}

/// Draws the camera of recording `artifact_id` with `live` from now on.
pub(crate) fn set_camera_effects(
  app: &AppHandle,
  artifact_id: u64,
  live: Option<Arc<LiveCameraEffects>>,
) {
  if let Some(state) = app.try_state::<RecordingPreviewPlayerState>() {
    if let Ok(manager) = state.0.lock() {
      let open = manager
        .sources
        .as_ref()
        .filter(|_| manager.artifact_id == Some(artifact_id));
      if let Some(sources) = open {
        *sources
          .camera_effects
          .write()
          .unwrap_or_else(|poisoned| poisoned.into_inner()) = live.clone();
        // While objects are picked the camera is drawn as recorded, and
        // with these once picking ends.
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(surface) = sources
          .preview_surface
          .as_ref()
          .filter(|_| sources.camera_picking_now().is_none())
        {
          if let Err(error) = surface.set_camera_effects(live) {
            eprintln!("Could not draw the camera's effects: {error}");
          }
        }
      }
    }
  }
  let _ = app.emit(CHANGED_EVENT, artifact_id);
}

/// Where the preview of the recording `artifact_id` stands, in milliseconds
/// of the recording, while it is open.
pub(crate) fn position_ms(app: &AppHandle, artifact_id: u64) -> Option<u64> {
  let state = app.try_state::<RecordingPreviewPlayerState>()?;
  let manager = state.0.lock().ok()?;
  (manager.artifact_id == Some(artifact_id)).then_some(manager.position_ms)
}
