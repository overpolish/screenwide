// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl PreviewPlayerManager {
  /// Re-presents the frame the pane already holds with the annotations the
  /// clips resolve to at `position_ms`, without touching the decoder. Reports
  /// whether there was a frame to redraw; a pane with nothing composed yet has
  /// to be restarted the ordinary way. Only the D3D11 panes can do this - the
  /// macOS workspace re-encodes from its retained scene instead. In One video
  /// the camera is drawn in the screen's pane, with its annotations over it,
  /// so a change on either redraws that pane.
  #[cfg(target_os = "windows")]
  pub(super) fn redraw_annotation_frame(&self, pane: u32, position_ms: u64) -> bool {
    let Some(sources) = self.sources.as_ref() else {
      return false;
    };
    let Some(surface) = sources.preview_surface.as_ref() else {
      return false;
    };
    let composition = sources
      .composition_settings
      .as_ref()
      .and_then(|settings| settings.read().ok().map(|settings| settings.clone()));
    let baked = composition
      .as_ref()
      .is_some_and(|composition| composition.bake_camera);
    let pane = if baked { 0 } else { pane };
    let Some(layout) = sources.layout.panes.get(pane as usize) else {
      return false;
    };
    let clips = sources
      .annotation_clips
      .read()
      .map(|clips| clips.clone())
      .unwrap_or_default();
    let ranges = sources
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let resolve = |track, picture| {
      crate::editor::annotations::timing::revealed_annotations(
        &clips,
        track,
        &ranges,
        position_ms,
        0.0,
        picture,
      )
    };
    let mut annotations = resolve(
      super::gesture::track(pane),
      (layout.source_width, layout.source_height),
    );
    if let Some(mut composition) = composition.filter(|_| baked) {
      let pictures = sources.annotation_pictures();
      composition.recording_output.primary.annotations = annotations;
      composition.recording_output.camera.annotations =
        resolve(AnnotationTrack::Camera, pictures[1]);
      sources.arrange_scene(&mut composition, position_ms, 0.0);
      super::super::annotation_preview::carry_camera_annotations(&mut composition, pictures);
      annotations = composition.recording_output.primary.annotations;
    }
    surface.redraw_recording_annotations(
      pane,
      &annotations,
      (layout.source_width, layout.source_height),
    )
  }

  #[cfg(not(target_os = "windows"))]
  pub(super) fn redraw_annotation_frame(&self, _pane: u32, _position_ms: u64) -> bool {
    false
  }
}
