// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The captions a preview session draws over the picture: made from the
//! saved edit as the session opens, then replaced each time the editor's
//! Captions lane makes them again.

use super::*;
use screenwide_text::captions::{CaptionMoment, CaptionTrack};
use screenwide_timeline::source_to_output_us;
use screenwide_timeline_edit::RecordingTimelineEdit;

pub(in crate::editor::recording_preview_player) type SharedCaptions =
  Arc<RwLock<Option<Arc<CaptionTrack>>>>;

/// The captions the saved `edit` shows, set in the session's output canvas,
/// or the recording's own `size` where a headless session sends none.
pub(super) fn saved_captions(
  recording: (&std::path::Path, &[RecordingAudioTrack], u64),
  edit: Option<&RecordingTimelineEdit>,
  settings: Option<&PreviewPlayerSettings>,
  size: (u32, u32),
) -> SharedCaptions {
  let (width, height) = settings.map_or(size, |settings| {
    let output = &settings.recording_output.primary;
    (output.width, output.height)
  });
  let frame = crate::editor::captions::frame_for(width, height);
  Arc::new(RwLock::new(edit.and_then(|edit| {
    crate::editor::captions::track_for(recording, edit, frame).map(Arc::new)
  })))
}

impl PlayerSources {
  /// The captions, and the moment of the edited video `source_ms` into the
  /// recording plays at, exposed over `frame_ms` as the frame's annotations
  /// are, for a frame to draw them at.
  pub(in crate::editor::recording_preview_player) fn captions_at(
    &self,
    source_ms: u64,
    frame_ms: f32,
  ) -> Option<(Arc<CaptionTrack>, CaptionMoment)> {
    let track = self
      .captions
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()?;
    let ranges = self
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let source_us = source_ms.saturating_mul(1_000);
    let output_us = if ranges.is_empty() {
      source_us
    } else {
      source_to_output_us(&ranges, source_us)?
    };
    let moment = CaptionMoment {
      at_ms: output_us as f64 / 1_000.0,
      exposure_ms: f64::from(frame_ms),
    };
    Some((track, moment))
  }
}

/// Draws `track` over the picture of recording `artifact_id` from now on,
/// and the paused frame again where it changed.
pub(crate) fn show_captions(app: &AppHandle, artifact_id: u64, track: Option<CaptionTrack>) {
  let Some(state) = app.try_state::<RecordingPreviewPlayerState>() else {
    return;
  };
  let Ok(mut manager) = state.0.lock() else {
    return;
  };
  let Some(sources) = manager
    .sources
    .as_ref()
    .filter(|_| manager.artifact_id == Some(artifact_id))
  else {
    return;
  };
  let track = track.map(Arc::new);
  {
    let mut shown = sources
      .captions
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if *shown == track {
      return;
    }
    *shown = track;
  }
  if !manager.is_playing {
    if let Err(error) = manager.restart(PlaybackMode::InteractiveStill) {
      eprintln!("Could not draw the captions: {error}");
    }
  }
}
