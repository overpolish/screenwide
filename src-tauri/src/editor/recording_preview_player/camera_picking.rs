// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects picked in the camera on the preview: while picking, the camera is
//! drawn as recorded, without its effects, so what the matte leaves out can
//! be clicked too, and the objects picked in the frame shown are marked.

use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Emitter, Manager};

use super::{PlayerSources, RecordingPreviewPlayerState};

/// Sent with a pick just made on the preview, which is chosen, as
/// `{ artifactId, id }`.
pub(crate) const CHOSEN_EVENT: &str = "editor://camera-object-chosen";
/// Sent with the object chosen once the preview has left the frame it was
/// picked in, as `{ artifactId, id }`.
const LEFT_EVENT: &str = "editor://camera-object-left";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CameraObjectEvent {
  pub(crate) artifact_id: u64,
  pub(crate) id: u32,
}

/// An object picked, and the span of the camera's time its frame shows for.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PickedSpan {
  pub(crate) id: u32,
  pub(crate) from_ms: u64,
  pub(crate) to_ms: u64,
  pub(crate) x: f32,
  pub(crate) y: f32,
  /// The line painted over it, or none where it was clicked.
  pub(crate) line: Vec<[f32; 2]>,
  pub(crate) keep: bool,
}

/// Picking objects in the camera of the recording `artifact_id`: every
/// object picked, and the one chosen.
#[derive(Clone)]
pub(crate) struct CameraPicking {
  pub(crate) app: AppHandle,
  pub(crate) artifact_id: u64,
  pub(crate) picked: Vec<PickedSpan>,
  pub(crate) chosen: Option<u32>,
}

pub(super) type SharedPicking = Arc<RwLock<Option<CameraPicking>>>;

impl PlayerSources {
  pub(super) fn camera_picking_now(&self) -> Option<CameraPicking> {
    self
      .camera_picking
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }

  /// The selection target the camera's picture is: the recording itself
  /// where it is of the camera alone, its own layer otherwise.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  fn camera_layer(&self) -> u32 {
    u32::from(self.primary_kind != crate::recording::PrimaryRecordingKind::Camera)
  }

  /// Marks the objects picked in the camera frame showing at camera time
  /// `camera_ms`, or none during playback, while objects are picked.
  ///
  /// The picking is held while they are handed over, so marks drawn for a
  /// frame and those sent for a new choice reach the preview in the order the
  /// picking changed: read first and handed over after, a frame's marks
  /// would put back the object chosen before the choice.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn show_picked(&self, camera_ms: Option<u64>) {
    let picking = self
      .camera_picking
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (Some(picking), Some(surface)) = (picking.as_ref(), &self.preview_surface) else {
      return;
    };
    let (dots, lines) = camera_ms.map_or_else(Default::default, |ms| picking.dots_at(ms));
    surface.set_object_picking(Some((self.camera_layer(), &dots, &lines)));
  }

  /// Lets the object chosen go once the still shown at camera time `ms` is
  /// not of the frame it was picked in, telling the editor. Only a still
  /// shown says so: the preview may not have reached the frame of an object
  /// just chosen elsewhere yet.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn let_go_elsewhere(&self, ms: u64) {
    let mut picking = self
      .camera_picking
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(picking) = picking.as_mut() else {
      return;
    };
    let Some(id) = picking.chosen else {
      return;
    };
    let shown = picking
      .picked
      .iter()
      .any(|pick| pick.id == id && (pick.from_ms..pick.to_ms).contains(&ms));
    if shown {
      return;
    }
    picking.chosen = None;
    let _ = picking.app.emit(
      LEFT_EVENT,
      CameraObjectEvent {
        artifact_id: picking.artifact_id,
        id,
      },
    );
  }
}

impl CameraPicking {
  /// The objects picked in the camera frame showing at camera time `ms`,
  /// and the points of the lines painted over them, in turn.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  fn dots_at(
    &self,
    ms: u64,
  ) -> (
    Vec<crate::editor::preview_platform::object_picking::PickedObjectDot>,
    Vec<[f32; 2]>,
  ) {
    let mut lines = Vec::new();
    let dots = self
      .picked
      .iter()
      .filter(|pick| (pick.from_ms..pick.to_ms).contains(&ms))
      .map(|pick| {
        let first = lines.len() as u32;
        lines.extend_from_slice(&pick.line);
        crate::editor::preview_platform::object_picking::PickedObjectDot {
          x: pick.x,
          y: pick.y,
          id: pick.id,
          removed: u32::from(!pick.keep),
          chosen: u32::from(self.chosen == Some(pick.id)),
          line_first: first,
          line_points: pick.line.len() as u32,
        }
      })
      .collect();
    (dots, lines)
  }
}

/// Picks objects in the camera of recording `artifact_id` as `picking` says,
/// or stops with `None`, the camera then drawn with its effects again.
pub(crate) fn set_camera_picking(
  app: &AppHandle,
  artifact_id: u64,
  picking: Option<CameraPicking>,
) {
  let Some(state) = app.try_state::<RecordingPreviewPlayerState>() else {
    return;
  };
  let Ok(manager) = state.0.lock() else {
    return;
  };
  let Some(sources) = manager
    .sources
    .as_ref()
    .filter(|_| manager.artifact_id == Some(artifact_id))
  else {
    return;
  };
  let was_picking = sources.camera_picking_now().is_some();
  let picking_now = picking.is_some();
  *sources
    .camera_picking
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = picking;
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(surface) = &sources.preview_surface {
    if was_picking != picking_now {
      let live = (!picking_now)
        .then(|| sources.camera_effects_now())
        .flatten();
      if let Err(error) = surface.set_camera_effects(live) {
        eprintln!("Could not draw the camera's effects: {error}");
      }
    }
    if picking_now {
      let shown = (!manager.is_playing).then_some(manager.position_ms);
      sources.show_picked(shown.map(|ms| camera_ms(sources, ms)));
    } else {
      surface.set_object_picking(None);
    }
  }
}

/// The camera's time showing at `position_ms` of the recording.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn camera_ms(sources: &PlayerSources, position_ms: u64) -> u64 {
  position_ms.min(
    sources
      .camera_duration_ms
      .unwrap_or(sources.duration_ms)
      .saturating_sub(1),
  )
}

/// Chooses the object `id`, a pick just made on the preview of recording
/// `artifact_id`, telling the editor.
pub(crate) fn choose_camera_object(app: &AppHandle, artifact_id: u64, id: u32) {
  if let Some(state) = app.try_state::<RecordingPreviewPlayerState>() {
    if let Ok(manager) = state.0.lock() {
      let open = manager
        .sources
        .as_ref()
        .filter(|_| manager.artifact_id == Some(artifact_id));
      if let Some(sources) = open {
        let mut picking = sources
          .camera_picking
          .write()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(picking) = picking.as_mut() {
          picking.chosen = Some(id);
        }
      }
    }
  }
  let _ = app.emit(CHOSEN_EVENT, CameraObjectEvent { artifact_id, id });
}
