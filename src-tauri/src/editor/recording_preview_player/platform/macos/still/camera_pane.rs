// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's own layer in Separate files, where the camera is a pane of
//! its own rather than drawn into the screen's composition.

use super::*;
use crate::editor::recording_preview_player::PreviewCompositionSettings;

/// The camera pane's layer: `camera`, the frame decoded `camera_ms` into the
/// camera, drawn at the pane's output scaled to `target_sizes`, with its
/// annotations moved onto the decoded frame. `None` while that output is
/// below the compositor's floor, before the webview has sent a real one.
pub(super) fn camera_pane_layer<'a>(
  composition: &PreviewCompositionSettings,
  sources: &PlayerSources,
  target_sizes: &[(u32, u32)],
  camera: &'a DecodedFrame,
  (camera_ms, screen_ms): (u64, u64),
) -> Option<RecordingWorkspaceLayer<'a>> {
  let own = &composition.recording_output.camera;
  let factor = super::super::still_decode::pane_factor(target_sizes, 1, own.width);
  let mut output = scaled_output(own, factor);
  if let Some(source) = sources.playback_layout.panes.get(1) {
    let metadata = camera.metadata();
    crate::editor::recording_preview_player::annotation_preview::remap_source(
      &mut output,
      (source.source_width, source.source_height),
      (metadata.width, metadata.height),
      own.width,
    );
  }
  if output.width < 64 || output.height < 64 {
    return None;
  }
  let (source, source_pixels) = match camera.rgba() {
    Some(source) => (Some(source), None),
    None => (
      None,
      camera.pixels().map(|pixels| (pixels, camera.dimensions())),
    ),
  };
  Some(RecordingWorkspaceLayer {
    pane_index: 1,
    source_token: (camera_ms << 2) | 1,
    source,
    source_pixels,
    settings: output,
    placement: NativeWorkspacePlacement::default(),
    seconds: screen_ms as f64 / 1_000.0,
    cursor: None,
    keyboard: None,
    camera: None,
    camera_pixels: None,
    overlay: None,
    camera_settings: None,
    clip_cursor_at_video_edge: false,
    foreground_only: false,
  })
}
