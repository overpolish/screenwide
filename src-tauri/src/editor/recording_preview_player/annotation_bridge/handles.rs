// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl PreviewPlayerManager {
  /// Takes an annotation tool in hand, or puts it down, in the native
  /// `ScreenwideAnnotationMode` the chrome is published with: nothing,
  /// hit-test the annotations already there, or also draw a new one on empty
  /// picture. A gesture in flight keeps the mode it began under. Changing the
  /// tool retires the halo, which the pointer may never move to retire; the
  /// next move halos whatever the new tool picks.
  ///
  /// Answers whether the layers are picked up under `tool`. A drawing tool
  /// picks none: the layers' targets only say which picture a fresh
  /// annotation starts on, and where the annotations of the layer not in hand
  /// are measured.
  pub(in crate::editor::recording_preview_player) fn set_annotation_tool(
    &mut self,
    tool: Option<&str>,
  ) -> bool {
    let mode = annotation_mode(tool);
    let picks_layers = drawing_kind(mode).is_none();
    if self.annotation.gesture.is_some() || self.annotation.group.is_some() {
      return picks_layers;
    }
    if mode != self.annotation.mode {
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      if let Some(surface) = self
        .sources
        .as_ref()
        .and_then(|sources| sources.preview_surface.as_ref())
      {
        surface.redraw_annotation_hover(None);
      }
    }
    self.annotation.mode = mode;
    picks_layers
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

  /// The frame under the pointer, for a highlight to select from. Only the
  /// screen pane has text to find. The first call for a frame starts its
  /// decode and answers `None`, as detection does; `wait` holds the pointer
  /// for a decode already on its way, which the end of a gesture does rather
  /// than commit the plain band it drew before the frame landed.
  pub(super) fn annotation_picture(
    &self,
    pane: u32,
    position_ms: u64,
    source: (u32, u32),
    wait: bool,
  ) -> Option<Arc<crate::editor::annotations::highlight::picture::HighlightPicture>> {
    use crate::editor::annotations::highlight::picture::{request_picture, HighlightPicture};
    let sources = self.sources.as_ref()?;
    if pane != 0 {
      return None;
    }
    let key = (self.artifact_id?, pane, position_ms);
    let cache = &self.annotation.pictures;
    if let Some(picture) = cache.picture(key) {
      return Some(picture);
    }
    let path = sources.screen_path.clone();
    let duration_ms = sources.duration_ms;
    request_picture(cache, key, move || {
      let frame = platform::source_frame_image(&path, position_ms, duration_ms).ok()?;
      HighlightPicture::new(Arc::new(frame), source)
    });
    wait
      .then(|| cache.wait(key, std::time::Duration::from_millis(400)))
      .flatten()
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
  /// What `pane` draws at the playhead: a gesture's working list while one
  /// holds that pane, and otherwise the clips live there.
  pub(in crate::editor::recording_preview_player) fn pane_annotations(
    &self,
    pane: u32,
  ) -> Vec<Annotation> {
    if let Some(gesture) = &self.annotation.gesture {
      if gesture.pane == pane {
        return gesture.working.clone();
      }
    }
    let Some(sources) = self.sources.as_ref() else {
      return Vec::new();
    };
    sources
      .annotation_clips
      .read()
      .map(|clips| {
        active_annotations(
          &clips,
          super::gesture::track(pane),
          self.position_ms.min(sources.duration_ms.saturating_sub(1)),
        )
      })
      .unwrap_or_default()
  }

  pub(in crate::editor::recording_preview_player) fn annotation_targets(
    &self,
  ) -> Vec<(u32, Annotation)> {
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
    let Some(composition) = self.drawn_composition() else {
      return;
    };
    let mut handles = Vec::new();
    let mut paths = Vec::new();
    let in_hand = match self.annotation.selected.as_slice() {
      [id] => Some(id),
      _ => None,
    };
    let group = self.annotation_group();
    let mut selected = if in_hand.is_some() { -2 } else { -1 };
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let mut group_boxes = Vec::new();
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
        .position(|annotation| Some(&annotation.id) == in_hand)
      {
        selected = (handles.len() + index) as i32;
      }
      let source_size = (source.source_width, source.source_height);
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      if !group.is_empty() {
        group_boxes.extend(self.pane_group_boxes(
          pane as u32,
          &annotations,
          source_size,
          output.size_image_width(),
        ));
      }
      handles.extend(
        annotation_handles(
          &annotations,
          source_size,
          output.size_image_width(),
          &mut paths,
        )
        .into_iter()
        .zip(&annotations)
        .map(|(mut handle, annotation)| {
          handle.layer_id = pane as i32;
          if group.contains(&annotation.id) {
            handle.flags |= HANDLE_FLAG_GROUPED;
          }
          handle
        }),
      );
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    surface.set_annotation_group(&group_boxes);
    surface.set_annotation_layer(
      &handles,
      &paths,
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
