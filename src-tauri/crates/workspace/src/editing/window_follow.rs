// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The viewport carried to the window's size ahead of the page. The page
//! measures the viewport and sends it, which takes a round trip through the
//! webview; while a window's edge is dragged that leaves the picture and the
//! checkerboard a step behind the edge. A platform that hears of the
//! window's new size first hands it here, and the viewport grows or shrinks
//! with it at once: the page keeps the chrome round the viewport at a fixed
//! size, so the viewport changes as much as the window does. The panes are
//! fitted into it as the page fits them. The page's next layout, carried the
//! same way to the window's size by then, replaces the guess.
//!
//! Zoomed in or panned, a window resized leaves the picture where it is on
//! screen, at the size it is: an edge dragged in crops it on that side and
//! one dragged out shows more on that side. Fitted, the picture is fitted
//! into the new viewport.

use super::*;

#[derive(Default)]
pub(super) struct WindowFollow {
  /// The viewport as the page laid it out.
  page_viewport: Option<PreviewSurfaceRect>,
  /// The window's size, in points, the page measured that viewport in.
  page_window: Option<(f64, f64)>,
  /// The window's content size, in physical pixels, as the platform last
  /// reported it. Kept in pixels and turned into points when used: Windows
  /// reports the size as an editor opens, before the first layout has set
  /// the scale, and points worked out then would be off by the scale.
  window: Option<(u32, u32)>,
  /// Where the window's content area's corner is on screen, in physical
  /// pixels, as last reported.
  origin: Option<(i32, i32)>,
  /// The window's size in points, and its corner, as the viewport now laid
  /// out was laid out in.
  shown_window: Option<(f64, f64)>,
  shown_origin: Option<(i32, i32)>,
}

impl WorkspaceEditor {
  /// Carries the viewport to the window's new size, `size` physical pixels
  /// of content with its corner at `origin` on screen, before the page lays
  /// it out again. A window moved whole is handed here too, at the size it
  /// had: the picture moves with it, and the next resize measures from the
  /// window's new place.
  pub fn follow_window(&self, size: (u32, u32), origin: Option<(i32, i32)>) {
    // A minimized window reports no size; it lays out afresh when restored.
    if size.0 == 0 || size.1 == 0 {
      return;
    }
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    state.window_follow.window = Some(size);
    set_origin(&mut state, origin);
    if !reproject(&mut state) {
      return;
    }
    let active = state.editor_active;
    super::controls::place_viewport(&self.inner, &state, active);
    apply_workspace_transform(&self.inner, &mut state, false);
    // A layout in flight on another thread holds the batch open; the
    // viewport's place lands with its commit.
    if self.inner.batch_depth.load(Ordering::Acquire) > 0 {
      self.inner.commit_pending.store(true, Ordering::Release);
    }
  }
}

/// Notes where the window's content area's corner is on screen, in physical
/// pixels, where the platform could say.
pub(super) fn set_origin(state: &mut SurfaceState, origin: Option<(i32, i32)>) {
  if origin.is_some() {
    state.window_follow.origin = origin;
  }
}

/// Takes the viewport the page laid out in a window of `page_window` points,
/// and lays it over the window as it is now.
pub(super) fn set_page_viewport(
  state: &mut SurfaceState,
  rect: PreviewSurfaceRect,
  page_window: Option<(f64, f64)>,
) {
  state.window_follow.page_viewport = Some(rect);
  state.window_follow.page_window = page_window;
  reproject(state);
}

/// A pane's box as the page laid it out, carried into the viewport as it is
/// now.
pub(super) fn page_pane_rect(state: &SurfaceState, rect: PreviewSurfaceRect) -> PreviewSurfaceRect {
  match state.window_follow.page_viewport {
    Some(page) if !same_size(page, state.viewport) => refit(
      rect,
      canvas_fit(state, page),
      canvas_fit(state, state.viewport),
    ),
    _ => rect,
  }
}

/// Moves the viewport to where the page's lands in the window as it is now,
/// refitting the panes laid out in the old one. Answers whether it moved.
fn reproject(state: &mut SurfaceState) -> bool {
  let Some(page) = state.window_follow.page_viewport else {
    return false;
  };
  let grown = |now: f64, then: f64| {
    let by = now - then;
    // The page measures in whole points and the platform in pixels, so a
    // difference under a point is rounding, not a resize.
    if by.abs() < 1.0 {
      0.0
    } else {
      by
    }
  };
  let scale = state.scale.max(0.1);
  let native = state
    .window_follow
    .window
    .map(|(width, height)| (f64::from(width) / scale, f64::from(height) / scale));
  let (width, height) = match (native, state.window_follow.page_window) {
    (Some(now), Some(then)) => (
      (page.width + grown(now.0, then.0)).max(0.0),
      (page.height + grown(now.1, then.1)).max(0.0),
    ),
    _ => (page.width, page.height),
  };
  let viewport = PreviewSurfaceRect {
    width,
    height,
    ..page
  };
  let window = native.or(state.window_follow.page_window);
  let resized = match (state.window_follow.shown_window, window) {
    (Some(then), Some(now)) => grown(now.0, then.0) != 0.0 || grown(now.1, then.1) != 0.0,
    _ => false,
  };
  let old = state.viewport;
  // Where the picture's corner sat on screen before, as seen from the new
  // viewport's: how far the window's corner and the viewport in it moved.
  let moved = match (state.window_follow.shown_origin, state.window_follow.origin) {
    (Some(then), Some(now)) => (
      f64::from(then.0 - now.0) / scale,
      f64::from(then.1 - now.1) / scale,
    ),
    _ => (0.0, 0.0),
  };
  let shift = (moved.0 + old.x - viewport.x, moved.1 + old.y - viewport.y);
  let kept = (resized && state.frame_resize.is_none() && !is_fitted(state.workspace_transform))
    .then(|| displayed(state))
    .flatten();
  state.window_follow.shown_window = window;
  state.window_follow.shown_origin = state.window_follow.origin;
  if same_size(old, viewport) && old.x == viewport.x && old.y == viewport.y {
    return false;
  }
  // A Frame resize places the panes itself, from the gesture's start.
  if !same_size(old, viewport) && state.frame_resize.is_none() {
    let (from, to) = (canvas_fit(state, old), canvas_fit(state, viewport));
    for pane in state.panes.iter_mut().flatten() {
      pane.base_rect = refit(pane.base_rect, from, to);
    }
  }
  state.viewport = viewport;
  if let (Some(shown), Some(laid)) = (kept, laid_out(state)) {
    if shown.width > 0.0 && laid.width > 0.0 {
      let shown = PreviewSurfaceRect {
        x: shown.x + shift.0,
        y: shown.y + shift.1,
        ..shown
      };
      state.workspace_transform = WorkspaceTransform::placing(viewport, laid, shown);
    }
  }
  true
}

/// Whether the view is as the double-click puts it: fitted, unpanned.
fn is_fitted(transform: WorkspaceTransform) -> bool {
  const EPSILON: f64 = 1e-9;
  (transform.zoom - 1.0).abs() < EPSILON
    && transform.pan_x.abs() < EPSILON
    && transform.pan_y.abs() < EPSILON
}

/// Where the panes are on screen, in the viewport's points.
pub(super) fn displayed(state: &SurfaceState) -> Option<PreviewSurfaceRect> {
  let transform = state.workspace_transform;
  state
    .panes
    .iter()
    .flatten()
    .filter(|pane| pane.seen)
    .map(|pane| transform.apply(state.viewport, pane.base_rect))
    .reduce(union_rect)
}

/// Where the panes are laid out, before the view's zoom and pan.
fn laid_out(state: &SurfaceState) -> Option<PreviewSurfaceRect> {
  state
    .panes
    .iter()
    .flatten()
    .filter(|pane| pane.seen)
    .map(|pane| pane.base_rect)
    .reduce(union_rect)
}

fn same_size(a: PreviewSurfaceRect, b: PreviewSurfaceRect) -> bool {
  a.width == b.width && a.height == b.height
}

/// Where the page fits the canvas in `viewport`, relative to its corner: the
/// box `fitPreviewPane` gives. Without a canvas yet, a point at its centre.
fn canvas_fit(state: &SurfaceState, viewport: PreviewSurfaceRect) -> DisplayRect {
  let size = (viewport.width, viewport.height);
  match state.workspace_natural_size {
    Some((width, height)) => {
      rebase_display_fit_mode(
        size,
        DisplayRect::default(),
        (f64::from(width), f64::from(height)),
        FIT_GUTTER,
        state.workspace_allows_upscale,
      )
      .fit
    }
    None => DisplayRect {
      x: size.0 / 2.0,
      y: size.1 / 2.0,
      width: 1.0,
      height: 1.0,
    },
  }
}

/// `rect` laid out against the fit `from`, put in the same place against
/// `to`: scaled by as much as the fit was, about the fit's centre.
fn refit(rect: PreviewSurfaceRect, from: DisplayRect, to: DisplayRect) -> PreviewSurfaceRect {
  let scale = to.width / from.width.max(f64::EPSILON);
  let width = rect.width * scale;
  let height = rect.height * scale;
  let centre_x =
    to.x + to.width / 2.0 + (rect.x + rect.width / 2.0 - from.x - from.width / 2.0) * scale;
  let centre_y =
    to.y + to.height / 2.0 + (rect.y + rect.height / 2.0 - from.y - from.height / 2.0) * scale;
  PreviewSurfaceRect {
    x: centre_x - width / 2.0,
    y: centre_y - height / 2.0,
    width,
    height,
  }
}

#[cfg(test)]
mod tests;
