// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scenes, arranged into the native preview's compositions the way the
//! export arranges its frames.

use super::scene_reframe::SceneReframe;
use super::*;
use screenwide_export::scenes::arrange;
use screenwide_scenes::{validate_clips, RecordingSceneClip};

/// The scenes the preview draws: the recording's clips, the timeline they
/// are timed on, and the pane of the one under the playhead the crop tool is
/// reframing, drawn unzoomed so the crop window shows what it picks from.
#[derive(Clone, Default)]
pub(super) struct PreviewScenes {
  pub(super) clips: Arc<RwLock<Vec<RecordingSceneClip>>>,
  pub(super) ranges: Arc<RwLock<Vec<TimelineRange>>>,
  pub(super) reframe: Arc<RwLock<Option<SceneReframe>>>,
}

impl PreviewScenes {
  pub(super) fn new(clips: Vec<RecordingSceneClip>, ranges: Vec<TimelineRange>) -> Self {
    Self {
      clips: Arc::new(RwLock::new(clips)),
      ranges: Arc::new(RwLock::new(ranges)),
      reframe: Arc::default(),
    }
  }

  /// Where the frame `source_ms` into the recording plays on the timeline
  /// the clips are timed on.
  pub(super) fn timeline_ms(&self, source_ms: u64) -> u64 {
    let ranges = self
      .ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    screenwide_annotations::timing::FrameTime::on(&ranges, source_ms).timeline_ms
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
      .scenes
      .ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let timeline_ms = screenwide_annotations::timing::FrameTime::on(&ranges, source_ms).timeline_ms;
    let reframe = self.scenes.reframe.read().ok().and_then(|reframe| *reframe);
    let reframed = super::scene_reframe::reframed_clips(&clips, reframe, timeline_ms);
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
      .find(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)
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

  /// Draws the paused frame again after a change that only moves pictures
  /// already decoded: its scenes, or the screen's turn. The frame is
  /// recomposed from the cached sources where it can be; a decoder restart
  /// would land the picture well after the selection the webview moves with
  /// the edit.
  pub(super) fn redraw_paused_frame(&mut self) -> Result<(), String> {
    if !self.is_playing && !self.recompose_paused_still() {
      self.restart(PlaybackMode::InteractiveStill)?;
    }
    Ok(())
  }
}

/// Hands the preview the recording's scene clips and the timeline they are
/// timed on, as `playback_ranges` lay it out, redrawing a paused frame that
/// they change.
#[tauri::command]
pub async fn set_recording_preview_scenes(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  clips: Vec<RecordingSceneClip>,
  playback_ranges: Vec<RecordingPreviewPlaybackRange>,
) -> Result<(), String> {
  validate_clips(&clips)?;
  let ranges = super::animation_timeline_ranges(&playback_ranges);
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
  let mut current_ranges = sources
    .scenes
    .ranges
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  if *current == clips && *current_ranges == ranges {
    return Ok(());
  }
  *current = clips;
  *current_ranges = ranges;
  drop(current);
  drop(current_ranges);
  manager.redraw_paused_frame()
}
