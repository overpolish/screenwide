// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::PreviewSurfaceRect;
use super::ffi::{
  screenwide_preview_surface_begin_layout,
  screenwide_preview_surface_clear_workspace_transform_history,
  screenwide_preview_surface_finish_layout, screenwide_preview_surface_hide,
  screenwide_preview_surface_layout_recording_workspace,
  screenwide_preview_surface_layout_workspace, screenwide_preview_surface_set_viewport,
};
use super::native_types::NativeWorkspacePaneRect;
use super::RecordingPreviewSurface;

impl RecordingPreviewSurface {
  pub(crate) fn set_viewport(&self, rect: PreviewSurfaceRect, backdrop: [f64; 4]) {
    unsafe {
      screenwide_preview_surface_set_viewport(
        self.handle,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        backdrop[0],
        backdrop[1],
        backdrop[2],
        // Preserve the CSS stack's alpha so AppKit's window material shows
        // through the workspace, just as it does through the webview.
        backdrop[3],
      );
    }
  }
  pub(crate) fn begin_layout(&self) {
    unsafe {
      screenwide_preview_surface_begin_layout(self.handle);
    }
  }
  pub(crate) fn set_scale(&self, _scale: f64) {}
  /// Lays out one fixed drawable over the complete viewport while retaining
  /// the logical canvas rectangle used by the native pan/zoom transform.
  pub(crate) fn layout_workspace(
    &self,
    rect: PreviewSurfaceRect,
    natural_size: (u32, u32),
    defer_draw: bool,
  ) {
    unsafe {
      screenwide_preview_surface_layout_workspace(
        self.handle,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        f64::from(natural_size.0),
        f64::from(natural_size.1),
        i32::from(defer_draw),
      );
    }
  }
  pub(crate) fn layout_recording_workspace(
    &self,
    rect: PreviewSurfaceRect,
    natural_size: (u32, u32),
    panes: &[(u32, PreviewSurfaceRect)],
    defer_draw: bool,
  ) {
    let panes = panes
      .iter()
      .map(|(index, pane)| NativeWorkspacePaneRect {
        index: *index,
        x: pane.x,
        y: pane.y,
        width: pane.width,
        height: pane.height,
      })
      .collect::<Vec<_>>();
    unsafe {
      screenwide_preview_surface_layout_recording_workspace(
        self.handle,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        f64::from(natural_size.0),
        f64::from(natural_size.1),
        panes.as_ptr(),
        panes.len().try_into().unwrap_or(u32::MAX),
        i32::from(defer_draw),
      );
    }
  }
  pub(crate) fn finish_layout(&self) {
    unsafe {
      screenwide_preview_surface_finish_layout(self.handle);
    }
  }
  pub(crate) fn clear_workspace_transform_history(&self) {
    unsafe {
      screenwide_preview_surface_clear_workspace_transform_history(self.handle);
    }
  }
  pub(crate) fn hide(&self) {
    unsafe {
      screenwide_preview_surface_hide(self.handle);
    }
  }
}
