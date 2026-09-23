// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl PreviewPlayerManager {
  /// Re-presents the frame the pane already holds with the annotations the
  /// clips resolve to at `position_ms`, without touching the decoder. Reports
  /// whether there was a frame to redraw; a pane with nothing composed yet has
  /// to be restarted the ordinary way. Only the D3D11 panes can do this - the
  /// Metal workspace re-encodes from its retained scene instead.
  #[cfg(target_os = "windows")]
  pub(super) fn redraw_annotation_frame(&self, pane: u32, position_ms: u64) -> bool {
    let Some(sources) = self.sources.as_ref() else {
      return false;
    };
    let Some(surface) = sources.preview_surface.as_ref() else {
      return false;
    };
    let Some(layout) = sources.layout.panes.get(pane as usize) else {
      return false;
    };
    let annotations = crate::editor::annotations::timing::revealed_annotations(
      &sources
        .annotation_clips
        .read()
        .map(|clips| clips.clone())
        .unwrap_or_default(),
      super::gesture::track(pane),
      position_ms,
      0.0,
    );
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

impl PreviewPlayerManager {
  /// The elements detected in the frame under the pointer, for an arrow's tip
  /// to land on. Only the screen pane of a screen recording has any: a camera
  /// or a window of pure video has no UI to aim at, and a counter never uses
  /// them.
  ///
  /// The first call for a frame starts the detection and answers `None`; the
  /// pointer is never held for it. Once it lands, every later sample of the
  /// same drag - and every later drag over the same frame - reads the cached
  /// result.
  pub(super) fn annotation_anchors(
    &self,
    pane: u32,
    position_ms: u64,
    source: (u32, u32),
  ) -> Option<Arc<AnchorBoxes>> {
    let sources = self.sources.as_ref()?;
    if pane != 0 || sources.primary_kind != PrimaryRecordingKind::Screen {
      return None;
    }
    let key = (self.artifact_id?, pane, position_ms);
    if let Some(anchors) = self.annotation.anchors.anchors(key) {
      return anchors.matches(source).then_some(anchors);
    }
    let path = sources.screen_path.clone();
    let duration_ms = sources.duration_ms;
    request_anchors(&self.annotation.anchors, key, move || {
      match platform::source_frame_image(&path, position_ms, duration_ms) {
        Ok(frame) => detect_anchors(&frame.rgba, frame.width, frame.height),
        // A frame that cannot be decoded has no elements, which is cached like
        // any other answer so the decode is not attempted again per sample.
        Err(_) => AnchorBoxes::new(0, 0, Vec::new()),
      }
    });
    None
  }

  /// Publishes what the sample on screen snapped to. It goes out before the
  /// grips, because on Windows publishing those is what redraws the chrome.
  pub(super) fn publish_annotation_snap(&self, source: (u32, u32), result: &SnapResult) {
    if let Some(surface) = self
      .sources
      .as_ref()
      .and_then(|sources| sources.preview_surface.as_ref())
    {
      surface.set_annotation_snap_guides(annotation_snap(result, source));
    }
  }
}
impl PreviewPlayerManager {
  pub(super) fn annotation_targets(&self) -> Vec<(u32, Annotation)> {
    let Some(sources) = &self.sources else {
      return Vec::new();
    };
    (0..sources.playback_layout.panes.len().min(2) as u32)
      .flat_map(|pane| {
        self
          .pane_annotations(pane)
          .into_iter()
          .map(move |annotation| (pane, annotation))
      })
      .collect()
  }

  pub(in crate::editor::recording_preview_player) fn publish_annotation_handles(&self) {
    let Some(sources) = &self.sources else {
      return;
    };
    let Some(surface) = &sources.preview_surface else {
      return;
    };
    let Some(composition) = self.selection_composition() else {
      return;
    };
    let mut handles = Vec::new();
    let mut selected = if self.annotation.selected.is_some() {
      -2
    } else {
      -1
    };
    for pane in 0..sources.playback_layout.panes.len().min(2) {
      let source = &sources.playback_layout.panes[pane];
      let output = if pane == 1 {
        &composition.recording_output.camera
      } else {
        &composition.recording_output.primary
      };
      let annotations = self.pane_annotations(pane as u32);
      if let Some(index) = annotations
        .iter()
        .position(|annotation| Some(&annotation.id) == self.annotation.selected.as_ref())
      {
        selected = (handles.len() + index) as i32;
      }
      handles.extend(
        annotation_handles(
          &annotations,
          (source.source_width, source.source_height),
          output.image_width,
        )
        .into_iter()
        .map(|mut handle| {
          handle.layer_id = pane as i32;
          handle
        }),
      );
    }
    surface.set_annotation_layer(
      &handles,
      selected,
      if self.is_playing {
        0
      } else {
        self.annotation.mode
      },
      self.annotation.pane.map_or(-1, |pane| pane as i32),
    );
  }
}
