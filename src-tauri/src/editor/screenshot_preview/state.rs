// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use super::super::preview_platform::{workspace_editor::WorkspaceScene, RecordingPreviewSurface};
use super::super::ScreenshotWorkspaceOutputSettings;
#[cfg(target_os = "macos")]
use super::annotation::AnnotationHover;
#[cfg(target_os = "macos")]
use super::annotation_gesture::AnnotationGestureOverride;
use super::gesture::SelectionGestureOverride;
use crate::screenshots::CapturedImage;

#[derive(Default)]
pub(super) struct PreviewManager {
  /// Set while the arrow tool owns the pointer, so a React layout arriving
  /// mid-drag cannot replace the working copy the gesture is drawing into.
  #[cfg(target_os = "macos")]
  pub(super) annotation_gesture: Option<AnnotationGestureOverride>,
  /// The style the next fresh arrow is drawn in: whatever the editor's last
  /// annotation edit settled on. `None` until it has settled on anything, in
  /// which case the arrow tool's own first dress stands.
  #[cfg(target_os = "macos")]
  pub(super) annotation_defaults: Option<crate::editor::annotations::AnnotationStyle>,
  /// The arrow the pointer rests on, and how wide its halo has grown.
  #[cfg(target_os = "macos")]
  pub(super) annotation_hover: Option<AnnotationHover>,
  /// What the pointer does over the picture, from the tool React has in hand.
  #[cfg(target_os = "macos")]
  pub(super) annotation_mode: u32,
  /// The pane the arrow chrome is drawn against, which is the only one its
  /// grips and its hit tests know about.
  #[cfg(target_os = "macos")]
  pub(super) annotation_pane_index: Option<u32>,
  pub(super) has_layout: bool,
  pub(super) latest_session_id: u64,
  pub(super) output: Option<ScreenshotWorkspaceOutputSettings>,
  pub(super) pane_target_size: Option<(u32, u32)>,
  pub(super) react_output: Option<ScreenshotWorkspaceOutputSettings>,
  pub(super) session_id: Option<u64>,
  pub(super) sources: Vec<(u64, Arc<CapturedImage>)>,
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
    }
    #[cfg(target_os = "macos")]
    {
      self.annotation_defaults = None;
      self.annotation_gesture = None;
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
