// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::timeline_edit;

pub(super) fn finish_estimate(
  screen_video: u64,
  camera_video: u64,
  selected_audio: u64,
  bake_camera: bool,
  timeline: Option<&timeline_edit::TimelinePlan>,
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
