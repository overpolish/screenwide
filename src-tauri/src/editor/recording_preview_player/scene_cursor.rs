// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cursor and the keys pressed as the scenes show or hide them, which
//! every preview path draws by, as the export does.

use screenwide_annotations::timing::{cursor_in_scenes, keys_in_scenes, video_ms, FrameTime};

use super::scene_preview::PreviewScenes;

impl PreviewScenes {
  /// How much of the cursor shows on the frame `source_ms` into the
  /// recording, from nothing to whole, `shown` being the recording's own
  /// choice: as the scene or gap there says, faded as it arrives and leaves.
  pub(super) fn cursor_shown(&self, source_ms: u64, shown: bool) -> f32 {
    self.on_clock(source_ms, |clips, at| cursor_in_scenes(clips, shown, at))
  }

  /// How much of the keys pressed show on the frame `source_ms` into the
  /// recording, from nothing to whole: hidden by a blank scene or a gap,
  /// faded as either arrives and leaves.
  pub(super) fn keys_shown(&self, source_ms: u64) -> f32 {
    self.on_clock(source_ms, keys_in_scenes)
  }

  /// `read` of the clips drawn, gaps among them, and where the frame
  /// `source_ms` into the recording falls on their timeline.
  fn on_clock(
    &self,
    source_ms: u64,
    read: impl FnOnce(&[screenwide_scenes::RecordingSceneClip], (u64, Option<u64>)) -> f32,
  ) -> f32 {
    // Clips before ranges, the order `set_recording_preview_scenes` writes
    // them in: the other order deadlocks a frame drawn while the scenes change.
    let clips = self
      .clips
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let ranges = self
      .ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let at = (
      FrameTime::on(&ranges, source_ms).timeline_ms,
      video_ms(&ranges),
    );
    read(clips.drawn(), at)
  }
}
