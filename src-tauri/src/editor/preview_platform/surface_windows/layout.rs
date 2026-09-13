// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn set_viewport(&self, rect: PreviewSurfaceRect, backdrop: [f64; 4]) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.viewport = rect;
      let (x, right) = window::scaled_edges(rect.x, rect.width, state.scale);
      let (y, bottom) = window::scaled_edges(rect.y, rect.height, state.scale);
      self
        .inner
        .editor
        .set_frame(x, y, right - x, bottom - y, state.editor_active);
      self.inner.gpu.backdrop.set_geometry(rect, state.scale);
      if let Ok(mut ribbon) = self.inner.gpu.audio_ribbon.lock() {
        let _ = ribbon.set_viewport(rect, state.scale, backdrop);
      }
      if state.backdrop != backdrop
        && self
          .inner
          .gpu
          .backdrop
          .paint(&self.inner.gpu.context, backdrop)
          .is_ok()
      {
        state.backdrop = backdrop;
      }
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn begin_layout(&self) {
    if let Ok(mut state) = self.inner.state.lock() {
      for pane in state.panes.iter_mut().flatten() {
        pane.seen = false;
      }
    }
  }

  /// `defer_resize` holds the new pane geometry back until the re-composed
  /// frame for it presents, so rect and pixels reach the compositor together
  /// instead of the old buffer shifting into the new rect for a tick.
  pub(super) fn layout_pane(
    &self,
    state: &mut SurfaceState,
    index: u32,
    rect: PreviewSurfaceRect,
    defer_resize: bool,
  ) {
    let index = index as usize;
    if state.panes.len() <= index {
      state.panes.resize_with(index + 1, || None);
    }
    if state.panes[index].is_none() {
      let below = state.panes[..index]
        .iter()
        .rev()
        .flatten()
        .next()
        .map(|pane| pane.visual.clone());
      state.panes[index] = self.inner.gpu.pane(below.as_ref()).ok();
    }
    let scale = state.scale;
    let viewport = state.viewport;
    let transform = state.workspace_transform;
    // A live Frame resize or auto-fit Move owns the workspace geometry: the
    // DOM rect still describes the canvas the drag started from (or the one
    // React last heard about, a layout behind), so adopting it would stretch
    // the freshly composed canvas back into that box until the next native
    // sample - the screenshot path jittered between the two rects on every
    // event. The native re-flow already placed every pane, on the recording
    // workspace and the per-pane screenshot layout alike.
    let owns_geometry = state.frame_resize.is_some();
    let Some(pane) = state.panes[index].as_mut() else {
      return;
    };
    pane.seen = true;
    if !owns_geometry {
      pane.base_rect = rect;
    }
    let transformed = transform.apply(viewport, pane.base_rect);
    // With a present on the way the geometry waits for it, so the pane's rect
    // and its freshly composed pixels land in the same commit. A pure pan (no
    // present coming) applies at once - but never while the DOM rect's aspect
    // has outrun the buffer: stretching the old frame into a new-aspect rect
    // for one transaction is exactly the jitter this avoids. Deferred
    // geometry is published by the pane's next present or the batch flush.
    let deferred = defer_resize || self.inner.batch_depth.load(Ordering::Acquire) > 0;
    set_pane_geometry(pane, viewport, transformed, scale, deferred);
  }

  /// Lays out the recording panes as one retained workspace. DirectComposition
  /// still uses one visual per source (each visual shares the same transform),
  /// but callers submit the complete pane topology in one operation and the
  /// existing batch keeps geometry and pixels atomic.
  pub(crate) fn layout_recording_workspace(
    &self,
    _rect: PreviewSurfaceRect,
    natural_size: (u32, u32),
    panes: &[(u32, PreviewSurfaceRect)],
    defer_draw: bool,
  ) {
    self.layout_retained_workspace(natural_size, panes, defer_draw, true);
  }

  /// Screenshot markers use `fitPreviewPane`, which never enlarges content
  /// beyond one point per output pixel. Keep that baseline distinct from the
  /// always-fill recording workspace.
  pub(crate) fn layout_screenshot_workspace(
    &self,
    natural_size: (u32, u32),
    panes: &[(u32, PreviewSurfaceRect)],
    defer_draw: bool,
  ) {
    self.layout_retained_workspace(natural_size, panes, defer_draw, false);
  }

  pub(super) fn layout_retained_workspace(
    &self,
    natural_size: (u32, u32),
    panes: &[(u32, PreviewSurfaceRect)],
    defer_draw: bool,
    allow_upscale: bool,
  ) {
    let mut restored_zoom = None;
    if let Ok(mut state) = self.inner.state.lock() {
      state.workspace_allows_upscale = allow_upscale;
      if state.frame_resize.is_some() {
        // A live drag owns the canvas size too: the re-flow tracks it from
        // the gesture's starts, and the DOM is a layout behind.
      } else if state.frame_resize_committed {
        // The drag rebased the transform so the displayed pixels stayed put;
        // its committed layout must keep that transform and record it as the
        // one belonging to the canvas size the drag produced, so a later undo
        // and redo across this size restore the same zoom.
        state.frame_resize_committed = false;
        state.workspace_natural_size = Some(natural_size);
        let transform = state.workspace_transform;
        state.workspace_transforms.insert(natural_size, transform);
      } else if state.workspace_natural_size != Some(natural_size) {
        if let Some(transform) = state.workspace_transforms.get(&natural_size).copied() {
          if (state.workspace_transform.zoom - transform.zoom).abs() > 0.0001 {
            restored_zoom = Some(transform.zoom);
          }
          state.workspace_transform.zoom = transform.zoom;
          state.workspace_transform.pan_x = transform.pan_x;
          state.workspace_transform.pan_y = transform.pan_y;
        }
        state.workspace_natural_size = Some(natural_size);
      }
      for (index, rect) in panes {
        self.layout_pane(&mut state, *index, *rect, defer_draw);
      }
    }
    if let Some(zoom) = restored_zoom {
      emit_transform(&self.inner, zoom);
    }
  }

  /// Remembered transforms belong to one workspace topology. A recording
  /// switching between primary-only, split camera and baked camera must not
  /// restore a same-sized transform captured for a different set of frames.
  pub(crate) fn clear_workspace_transform_history(&self) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.workspace_transforms.clear();
    }
  }

  /// Applies one viewport-local transform to every pane while retaining their
  /// relative positions. Native Windows input will drive this directly.
  // Retained-workspace entry point; not wired on Windows yet.
  #[allow(dead_code)]
  pub(crate) fn set_workspace_transform(&self, pan_x: f64, pan_y: f64, zoom: f64) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    let zoom = zoom.clamp(0.1, maximum_editor_zoom(&state));
    state.workspace_transform = WorkspaceTransform { pan_x, pan_y, zoom };
    apply_workspace_transform(&self.inner, &mut state, false);
  }

  pub(crate) fn finish_layout(&self) {
    if let Ok(mut state) = self.inner.state.lock() {
      for pane in state.panes.iter_mut().flatten().filter(|pane| !pane.seen) {
        pane.hide();
        // A hidden pane has nothing stale to show; drop leftovers so a later
        // flush cannot resurrect it at a parked offset.
        pane.pending_geometry = false;
        pane.pending_present = false;
      }
      draw_selection(&self.inner, &state);
      // An open batch commits for everything on its flush; a second
      // commit-and-wait here would add a display tick of latency to every
      // layout that presents in the same invoke.
      if self.inner.batch_depth.load(Ordering::Acquire) > 0 {
        return;
      }
      if unsafe { self.inner.gpu.composition.Commit() }.is_ok() {
        // Commit is otherwise only queued. Waiting here prevents rapid DOM
        // pans from building a DirectComposition transaction backlog in which
        // the OSCs visibly outrun the video pane. The frontend already keeps
        // only the newest layout while this one reaches the compositor.
        let _ = unsafe { self.inner.gpu.composition.WaitForCommitCompletion() };
      }
    }
  }
}
