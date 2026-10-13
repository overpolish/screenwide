// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::EditorState;

/// What one estimate is kept under: the recording, which of its baked
/// layers and which file it is, its compression and its scale.
pub(super) type EstimateKey = (u64, u8, u8, u16);

/// The estimate kept under `key`, if one was made.
pub(super) fn kept_estimate(state: &EditorState, key: &EstimateKey) -> Option<u64> {
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(key)
    .copied()
}

/// Keeps `bytes` as the estimate under `key`, answering it.
pub(super) fn keep_estimate(state: &EditorState, key: EstimateKey, bytes: u64) -> u64 {
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .insert(key, bytes);
  bytes
}

pub(super) fn finish_estimate(
  screen_video: u64,
  camera_video: u64,
  selected_audio: u64,
  bake_camera: bool,
  timeline: Option<&screenwide_timeline_edit::TimelinePlan>,
  duration_ms: u64,
) -> u64 {
  // MP4's tables are small but real. Half a percent plus its fixed headers
  // keeps the estimate honest without pretending CRF can predict exact size.
  let screen_video = if bake_camera {
    screen_video.saturating_add(screen_video / 12)
  } else {
    screen_video
  };
  let media = screen_video
    .saturating_add(camera_video)
    .saturating_add(selected_audio);
  let media = timeline.map_or(media, |timeline| {
    media.saturating_mul(timeline.duration_ms()) / duration_ms.max(1)
  });
  media.saturating_add(media / 200).saturating_add(4_096)
}
