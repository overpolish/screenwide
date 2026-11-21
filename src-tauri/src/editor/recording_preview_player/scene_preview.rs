// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scenes, arranged into the native preview's compositions the way the
//! export arranges its frames.

use super::gap_preview::SceneClips;
use super::scene_reframe::SceneCrop;
use super::*;
use screenwide_export::scenes::arrange_following;
use screenwide_scenes::{FacePath, GapScene, RecordingSceneClip};

/// The scenes the preview draws: the clips and the gaps drawn among them, the
/// timeline they are timed on, how long the recording is, past which its gaps
/// lie, where the face is over the camera for the clips that follow it, the
/// pane of the one under the playhead the crop tool is reframing, drawn
/// unzoomed for the crop window, and the selection over a scene's pane.
#[derive(Clone, Default)]
pub(super) struct PreviewScenes {
  pub(super) clips: Arc<RwLock<SceneClips>>,
  pub(super) ranges: Arc<RwLock<Vec<TimelineRange>>>,
  pub(super) source_duration_us: u64,
  pub(super) face_path: Arc<RwLock<Option<Arc<FacePath>>>>,
  pub(super) crop: Arc<RwLock<SceneCrop>>,
  pub(super) selection: Arc<Mutex<super::scene_selection::DrawnSelection>>,
}

impl PreviewScenes {
  pub(super) fn new(
    (clips, gaps): (Vec<RecordingSceneClip>, Vec<GapScene>),
    ranges: Vec<TimelineRange>,
    source_duration_ms: u64,
    face_path: Option<Arc<FacePath>>,
  ) -> Self {
    Self {
      clips: Arc::new(RwLock::new(SceneClips::new(clips, gaps))),
      ranges: Arc::new(RwLock::new(ranges)),
      source_duration_us: source_duration_ms.saturating_mul(1_000),
      face_path: Arc::new(RwLock::new(face_path)),
      crop: Arc::default(),
      selection: Arc::default(),
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
/// frame covers, zero for a still; `face` is where the face is on the frame,
/// for a scene that follows it.
pub(super) fn arrange_composition(
  composition: &mut PreviewCompositionSettings,
  clips: &[RecordingSceneClip],
  ranges: &[TimelineRange],
  (source_ms, frame_ms): (u64, f32),
  camera: Option<(u32, u32)>,
  face: Option<(f64, f64)>,
) -> bool {
  let camera = camera.filter(|_| composition.bake_camera);
  arrange_following(
    clips,
    ranges,
    (source_ms, frame_ms),
    &mut composition.recording_output.primary,
    camera.map(|size| (&mut composition.camera_overlay, size)),
    face,
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
    let crop = self
      .scenes
      .crop
      .read()
      .map(|crop| *crop)
      .unwrap_or_default();
    let reframe = crop.pane;
    let camera = self.camera_source_size();
    // The camera follows the face while it is cropped too: the editor places
    // it from the clips and the same face (`useFacePath`), so its crop window
    // lies on the picture as the screen's does.
    let face = self
      .scenes
      .face_path
      .read()
      .ok()
      .and_then(|path| path.as_ref().map(|path| path.at(source_ms as f64)));
    let base = composition.recording_output.primary.clone();
    // What the crop tool opens out: a preset it holds where its panes stood,
    // else the scenes as they are. While the pane opens or closes the frame
    // is drawn part of the way between the scenes laid out and that, so
    // putting the tool away closes the picture onto where the scene lays it.
    let held = reframe
      .and_then(|_| clips.held_drawn())
      .unwrap_or(clips.drawn());
    let opening = reframe.filter(|_| crop.open < 1.0);
    let mut opened = opening.map(|_| composition.clone());
    let placed = arrange_composition(
      composition,
      if opening.is_some() {
        clips.drawn()
      } else {
        held
      },
      &ranges,
      (source_ms, frame_ms),
      camera,
      face,
    );
    if placed {
      super::scene_reframe::ease_screen_turn(
        &mut composition.recording_output.primary,
        crop.screen_turn,
      );
    }
    let Some(reframe) = reframe.filter(|_| placed) else {
      return placed;
    };
    let in_place = clips
      .own()
      .iter()
      .find(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)
      .is_some_and(|clip| super::scene_reframe::crops_in_place(clip, reframe));
    let pane = (reframe, in_place, camera);
    match opened.as_mut() {
      None => super::scene_reframe::open_pane(composition, pane, &base),
      Some(opened) => {
        arrange_composition(opened, held, &ranges, (source_ms, frame_ms), camera, face);
        super::scene_reframe::ease_screen_turn(
          &mut opened.recording_output.primary,
          crop.screen_turn,
        );
        super::scene_reframe::open_pane(opened, pane, &base);
        super::scene_reframe::ease_open(composition, opened, crop.open);
      }
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
      .and_then(|sources| {
        sources
          .scenes
          .clips
          .read()
          .ok()
          .map(|clips| clips.own().to_vec())
      })
      .unwrap_or_default()
  }

  /// Draws `clips` from now on, as a gesture reframes one of them.
  pub(super) fn set_scene_clips(&self, clips: Vec<RecordingSceneClip>) -> Result<(), String> {
    let sources = self
      .sources
      .as_ref()
      .ok_or_else(|| "The recording preview player is not open".to_owned())?;
    sources
      .scenes
      .clips
      .write()
      .map_err(|_| "The scenes are unavailable".to_owned())?
      .set(clips, None);
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
