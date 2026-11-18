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

use super::*;

#[derive(Default)]
pub(super) struct WindowFollow {
  /// The viewport as the page laid it out.
  page_viewport: Option<PreviewSurfaceRect>,
  /// The window's size, in points, the page measured that viewport in.
  page_window: Option<(f64, f64)>,
  /// The window's size, in points, as the platform last reported it.
  window: Option<(f64, f64)>,
}

impl WorkspaceEditor {
  /// Carries the viewport to the window's new size, `size` physical pixels
  /// of content, before the page lays it out again.
  pub fn follow_window(&self, size: (u32, u32)) {
    // A minimized window reports no size; it lays out afresh when restored.
    if size.0 == 0 || size.1 == 0 {
      return;
    }
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    let scale = state.scale.max(0.1);
    state.window_follow.window = Some((f64::from(size.0) / scale, f64::from(size.1) / scale));
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
  let (width, height) = match (state.window_follow.window, state.window_follow.page_window) {
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
  let old = state.viewport;
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
  true
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
mod tests {
  use super::*;

  fn rect(x: f64, y: f64, width: f64, height: f64) -> PreviewSurfaceRect {
    PreviewSurfaceRect {
      height,
      width,
      x,
      y,
    }
  }

  fn rounded(rect: PreviewSurfaceRect) -> [f64; 4] {
    [rect.x, rect.y, rect.width, rect.height].map(|value| (value * 1000.0).round() / 1000.0)
  }

  #[test]
  fn a_grown_window_grows_the_viewport_and_refits_the_page_pane() {
    let mut state = SurfaceState {
      workspace_natural_size: Some((2000, 1000)),
      ..SurfaceState::default()
    };
    // The page laid a 1000 by 600 viewport out in a 1200 by 800 window, the
    // canvas fitted into it inside the gutter.
    set_page_viewport(
      &mut state,
      rect(0.0, 40.0, 1000.0, 600.0),
      Some((1200.0, 800.0)),
    );
    let page_pane = rect(8.0, 54.0, 984.0, 492.0);
    assert_eq!(
      rounded(page_pane_rect(&state, page_pane)),
      rounded(page_pane)
    );

    // The window is 100 points wider before the page lays out again, and a
    // fraction of a point taller, which is rounding.
    state.window_follow.window = Some((1300.0, 800.4));
    assert!(reproject(&mut state));

    assert_eq!(rounded(state.viewport), [0.0, 40.0, 1100.0, 600.0]);
    assert_eq!(
      rounded(page_pane_rect(&state, page_pane)),
      [8.0, 29.0, 1084.0, 542.0]
    );
  }
}
