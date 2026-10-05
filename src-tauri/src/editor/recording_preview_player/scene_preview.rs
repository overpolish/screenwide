// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scenes, arranged into the native preview's compositions the way the
//! export arranges its frames.

use super::scene_reframe::SceneReframe;
use super::*;
use crate::editor::preview_platform::SelectionGestureOperation;
use crate::editor::scenes::{arrange, camera_target, validate_clips, RecordingSceneClip};

/// The scenes the preview draws: the recording's clips, and the pane of the
/// one under the playhead the crop tool is reframing, which is drawn unzoomed
/// so the crop window shows what it picks from.
#[derive(Clone, Default)]
pub(super) struct PreviewScenes {
  pub(super) clips: Arc<RwLock<Vec<RecordingSceneClip>>>,
  pub(super) reframe: Arc<RwLock<Option<SceneReframe>>>,
}

impl PreviewScenes {
  pub(super) fn new(clips: Vec<RecordingSceneClip>) -> Self {
    Self {
      clips: Arc::new(RwLock::new(clips)),
      reframe: Arc::default(),
    }
  }
}

/// Arranges `composition` as `clips` have it `source_ms` into the recording,
/// timed on `ranges`, answering whether a scene placed it. The camera is only
/// arranged where it is drawn into the picture, so a scene that places one
/// stands idle while the camera is its own pane or there is none. `camera` is
/// the camera's source size; `frame_ms` is how much source time the drawn
/// frame covers, zero for a still.
pub(super) fn arrange_composition(
  composition: &mut PreviewCompositionSettings,
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  (source_ms, frame_ms): (u64, f32),
  camera: Option<(u32, u32)>,
) -> bool {
  let camera = camera.filter(|_| composition.bake_camera);
  arrange(
    clips,
    ranges,
    source_ms,
    frame_ms,
    &mut composition.recording_output.primary,
    camera.map(|size| (&mut composition.camera_overlay, size)),
  )
}

impl PlayerSources {
  /// The camera's source size, where the recording has a camera.
  pub(super) fn camera_source_size(&self) -> Option<(u32, u32)> {
    self.camera_path.as_ref()?;
    self
      .playback_layout
      .panes
      .get(1)
      .map(|pane| (pane.source_width, pane.source_height))
  }

  /// Arranges `composition` as this recording's scenes have it `source_ms`
  /// in, answering whether a scene placed it. `frame_ms` is how much source
  /// time the drawn frame covers, zero for a still. A pane the crop tool is
  /// working on is drawn the way `scene_reframe` describes.
  pub(super) fn arrange_scene(
    &self,
    composition: &mut PreviewCompositionSettings,
    source_ms: u64,
    frame_ms: f32,
  ) -> bool {
    let Ok(clips) = self.scenes.clips.read() else {
      return false;
    };
    let ranges = self
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let reframe = self.scenes.reframe.read().ok().and_then(|reframe| *reframe);
    let reframed = super::scene_reframe::reframed_clips(&clips, reframe, source_ms);
    let camera = self.camera_source_size();
    let base = composition.recording_output.primary.clone();
    let placed = arrange_composition(
      composition,
      reframed.as_deref().unwrap_or(&clips),
      &ranges,
      (source_ms, frame_ms),
      camera,
    );
    let Some(reframe) = reframe.filter(|_| placed) else {
      return placed;
    };
    let in_place = clips
      .iter()
      .find(|clip| clip.start_ms <= source_ms && source_ms < clip.end_ms)
      .is_some_and(|clip| super::scene_reframe::crops_in_place(clip, reframe));
    match (reframe, camera.filter(|_| composition.bake_camera)) {
      (SceneReframe::Screen, _) if in_place => {
        super::scene_reframe::show_screen_picture(&mut composition.recording_output.primary, &base)
      }
      (SceneReframe::Camera, Some(camera)) if in_place => {
        super::scene_reframe::show_camera_picture(&mut composition.camera_overlay, camera)
      }
      (SceneReframe::Camera, Some(camera)) => {
        super::scene_reframe::show_whole_camera(&mut composition.camera_overlay, camera)
      }
      _ => {}
    }
    placed
  }
}

impl PreviewPlayerManager {
  /// The composition the paused frame is drawn in: the stored one, arranged
  /// as the scenes have it at the playhead.
  pub(super) fn arranged_composition(
    &self,
    composition: &PreviewCompositionSettings,
  ) -> PreviewCompositionSettings {
    let mut arranged = composition.clone();
    if let Some(sources) = &self.sources {
      sources.arrange_scene(&mut arranged, self.position_ms, 0.0);
    }
    arranged
  }

  /// The stored composition as the paused frame draws it, which annotation
  /// handles are sized and placed by.
  pub(super) fn drawn_composition(&self) -> Option<PreviewCompositionSettings> {
    self
      .selection_composition()
      .map(|composition| self.arranged_composition(&composition))
  }

  /// What a gesture does while the paused frame is inside a scene. A custom
  /// scene's boxes move and resize with the panes in them. Any other scene
  /// owns where the panes sit, so a move or a resize only reaches the
  /// camera's picture: dragged, it pans inside its box, and resized, it
  /// zooms, which reframes the camera of the clip under the playhead. `clips`
  /// are the clips as the gesture found them, which its deltas are measured
  /// from. `None` outside every scene and for the gestures a scene leaves as
  /// they are, a corner radius or the canvas's frame; `Some(None)` for a
  /// gesture a scene leaves with nothing to do; else the clips the gesture
  /// leaves.
  pub(super) fn scene_gesture(
    &self,
    snapshot: &PreviewCompositionSettings,
    clips: &[RecordingSceneClip],
    (layer_id, operation): (u32, SelectionGestureOperation),
    scale: f64,
    delta: (f64, f64),
  ) -> Option<Option<Vec<RecordingSceneClip>>> {
    let scale = match operation {
      SelectionGestureOperation::Move => 1.0,
      SelectionGestureOperation::Resize | SelectionGestureOperation::Radius => scale,
      _ => return None,
    };
    let sources = self.sources.as_ref()?;
    let mut arranged = snapshot.clone();
    if !sources.arrange_scene(&mut arranged, self.position_ms, 0.0) {
      return None;
    }
    // A radius gesture reports the radius it reaches as its scale.
    if operation == SelectionGestureOperation::Radius {
      return super::scene_pane_gesture::radius_gesture(
        clips,
        self.position_ms,
        snapshot.bake_camera,
        layer_id,
        scale,
      )
      .map(Some);
    }
    if let Some(next) = super::scene_pane_gesture::custom_box_gesture(
      clips,
      self.position_ms,
      snapshot.bake_camera,
      layer_id,
      (delta, scale),
    ) {
      return next.map(Some);
    }
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own; the screen's are swallowed.
    let camera = sources
      .camera_source_size()
      .filter(|_| snapshot.bake_camera);
    let camera = match (camera, layer_id) {
      (Some(camera), 1) => camera,
      (None, 1) => return None,
      _ => return Some(None),
    };
    let output = &snapshot.recording_output.primary;
    let Some((index, frame, framing)) = camera_target(
      clips,
      self.position_ms,
      output,
      &snapshot.camera_overlay,
      camera,
    ) else {
      return Some(None);
    };
    // The gesture's deltas are shares of the canvas it was made on.
    let canvas = crate::editor::preview_workspace_model::output_canvas(output);
    let mut next = clips.to_vec();
    next[index].camera = Some(framing.reframed(
      frame,
      f64::from(camera.0) / f64::from(camera.1.max(1)),
      (delta.0 * canvas.0, delta.1 * canvas.1),
      scale,
    ));
    Some(Some(next))
  }

  /// The scene clips the preview draws now, which a gesture starts from.
  pub(super) fn scene_clips(&self) -> Vec<RecordingSceneClip> {
    self
      .sources
      .as_ref()
      .and_then(|sources| sources.scenes.clips.read().ok().map(|clips| clips.clone()))
      .unwrap_or_default()
  }

  /// Draws `clips` from now on, as a gesture reframes one of them.
  pub(super) fn set_scene_clips(&self, clips: Vec<RecordingSceneClip>) -> Result<(), String> {
    let sources = self
      .sources
      .as_ref()
      .ok_or_else(|| "The recording preview player is not open".to_owned())?;
    *sources
      .scenes
      .clips
      .write()
      .map_err(|_| "The scenes are unavailable".to_owned())? = clips;
    Ok(())
  }
}

/// Hands the preview the recording's scene clips, redrawing a paused frame
/// that they change. A scene only moves pictures that are already decoded,
/// so the paused frame is recomposed from the cached sources; a decoder
/// restart would land the picture well after the selection the webview moves
/// with the same edit.
#[tauri::command]
pub async fn set_recording_preview_scenes(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingSceneClip>,
) -> Result<(), String> {
  validate_clips(&clips)?;
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let sources = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?;
  let mut current = sources
    .scenes
    .clips
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  if *current == clips {
    return Ok(());
  }
  *current = clips;
  drop(current);
  if !manager.is_playing && !manager.recompose_paused_still() {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}
