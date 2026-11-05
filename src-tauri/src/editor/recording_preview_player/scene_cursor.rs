// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cursor as the scenes show or hide it, which every preview path draws
//! by, as the export does.

use screenwide_annotations::timing::{cursor_in_scenes, video_ms, FrameTime};

use super::scene_preview::PreviewScenes;

impl PreviewScenes {
  /// How much of the cursor shows on the frame `source_ms` into the
  /// recording, from nothing to whole, `shown` being the recording's own
  /// choice: as the scene there says, faded as it arrives and leaves.
  pub(super) fn cursor_shown(&self, source_ms: u64, shown: bool) -> f32 {
    let ranges = self
      .ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let clips = self
      .clips
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let timeline_ms = FrameTime::on(&ranges, source_ms).timeline_ms;
    cursor_in_scenes(&clips, shown, (timeline_ms, video_ms(&ranges)))
  }
}
