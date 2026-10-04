// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use super::super::preview_platform::{workspace_editor::WorkspaceScene, RecordingPreviewSurface};
use super::super::ScreenshotWorkspaceOutputSettings;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::annotation::AnnotationHover;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::annotation_gesture::AnnotationGestureOverride;
use super::gesture::SelectionGestureOverride;
use crate::screenshots::{Capture, CapturedImage};

/// One captured image the preview draws, with what its layer settings need
/// of the item it came from.
#[derive(Clone)]
pub(super) struct PreviewSource {
  pub(super) id: u64,
  pub(super) image: Arc<CapturedImage>,
  /// What the image was captured at, which its redactions and annotation
  /// sizes are measured against.
  pub(super) capture: Capture,
}

#[derive(Default)]
pub(super) struct PreviewManager {
  /// Set while the arrow tool owns the pointer, so a React layout arriving
  /// mid-drag cannot replace the working copy the gesture is drawing into.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_gesture: Option<AnnotationGestureOverride>,
  /// The annotations chosen together being carried by a press on the picture.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_group: Option<super::annotation_group::GroupDrag>,
  /// A marquee band being drawn over the picture.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_band: Option<super::annotation_choice::Band>,
  /// Every annotation React has chosen on the layer, as its last layout said.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_selected: Vec<String>,
  /// Set while a text box is being typed into: the pane's working copy is the
  /// manager's until the typing ends, the way it is through a drag.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_text: Option<super::annotation_text::TextSession>,
  /// The latest pane's detected UI elements, for an arrow's tip to land on.
  /// Behind its own lock because the detection that fills it runs on a
  /// blocking thread and must never wait for the manager.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_anchor_cache: Arc<crate::editor::annotations::snap::AnchorCache>,
  /// The style the next fresh arrow is drawn in: whatever the editor's last
  /// annotation edit settled on. `None` until it has settled on anything, in
  /// which case the arrow tool's own first dress stands.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_defaults: Option<crate::editor::annotations::AnnotationStyle>,
  /// What a fresh annotation is made with beyond its dress: where a fresh
  /// counter's tail points, whatever the last counter was turned to, so a
  /// row of them is dropped aiming the same way, and the picture the last
  /// sticker was given. Each absent until it has been settled.
  pub(super) annotation_fresh: crate::editor::annotations::edit::FreshAnnotation,
  /// The arrow the pointer rests on, and how wide its halo has grown.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_hover: Option<AnnotationHover>,
  /// What the pointer does over the picture, from the tool React has in hand.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_mode: u32,
  /// The pane the arrow chrome is drawn against, which is the only one its
  /// grips and its hit tests know about.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) annotation_pane_index: Option<u32>,
  pub(super) has_layout: bool,
  pub(super) latest_session_id: u64,
  pub(super) output: Option<ScreenshotWorkspaceOutputSettings>,
  pub(super) pane_target_size: Option<(u32, u32)>,
  pub(super) react_output: Option<ScreenshotWorkspaceOutputSettings>,
  pub(super) session_id: Option<u64>,
  pub(super) sources: Vec<PreviewSource>,
  pub(super) surface: Option<Arc<RecordingPreviewSurface>>,
  pub(super) selection_gesture: Option<SelectionGestureOverride>,
  pub(super) workspace_scene: Option<WorkspaceScene>,
}

impl PreviewManager {
  pub(super) fn require_session(&self, session_id: u64) -> Result<(), String> {
    (self.session_id == Some(session_id))
      .then_some(())
      .ok_or_else(|| "That screenshot preview session is no longer active".to_owned())
  }

  pub(super) fn stop(&mut self) {
    if let Some(surface) = self.surface.as_ref() {
      surface.hide();
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      surface.end_annotation_text();
      // A save suspends the editor and the session ends before React lifts
      // it; the Windows editor window keeps the flag and would stay hidden
      // for every later session. After `hide`, lifting it shows nothing.
      #[cfg(target_os = "windows")]
      surface.set_editor_suspended(false);
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
      self.annotation_anchor_cache.clear();
      self.annotation_defaults = None;
      self.annotation_fresh = Default::default();
      self.annotation_gesture = None;
      self.annotation_group = None;
      self.annotation_band = None;
      self.annotation_selected = Vec::new();
      self.annotation_text = None;
      self.annotation_hover = None;
      self.annotation_mode = 0;
      self.annotation_pane_index = None;
    }
    self.has_layout = false;
    self.output = None;
    self.pane_target_size = None;
    self.react_output = None;
    self.session_id = None;
    self.sources.clear();
    self.surface = None;
    self.selection_gesture = None;
    self.workspace_scene = None;
  }
}

#[derive(Default)]
pub struct ScreenshotPreviewState(pub(super) Mutex<PreviewManager>);
