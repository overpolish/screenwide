// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
    scale: 2.0,
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

  // The window is 100 points wider before the page lays out again, and
  // half a point taller, which is rounding.
  state.window_follow.window = Some((2600, 1601));
  assert!(reproject(&mut state));

  assert_eq!(rounded(state.viewport), [0.0, 40.0, 1100.0, 600.0]);
  assert_eq!(
    rounded(page_pane_rect(&state, page_pane)),
    [8.0, 29.0, 1084.0, 542.0]
  );
}

#[test]
fn a_size_reported_before_the_scale_is_set_is_read_at_the_scale_laid_out_with() {
  // Windows reports the window's size as an editor opens, while the scale
  // is still the default.
  let mut state = SurfaceState {
    workspace_natural_size: Some((2000, 1000)),
    ..SurfaceState::default()
  };
  state.window_follow.window = Some((2400, 1600));
  // The first layout sets the scale, then lays out in that same window.
  state.scale = 2.0;
  set_page_viewport(
    &mut state,
    rect(0.0, 40.0, 1000.0, 600.0),
    Some((1200.0, 800.0)),
  );

  assert_eq!(rounded(state.viewport), [0.0, 40.0, 1000.0, 600.0]);
}

/// A page laid out at 1000 by 600 in a 1200 by 800 window whose corner is
/// 100 points into the screen, at twice the scale, its canvas fitted in
/// the gutter and shown through `transform`.
fn laid_out(transform: WorkspaceTransform) -> SurfaceState {
  let mut state = SurfaceState {
    scale: 2.0,
    workspace_natural_size: Some((2000, 1000)),
    ..SurfaceState::default()
  };
  set_origin(&mut state, Some((200, 200)));
  set_page_viewport(
    &mut state,
    rect(0.0, 40.0, 1000.0, 600.0),
    Some((1200.0, 800.0)),
  );
  state.panes = vec![Some(Pane {
    base_rect: rect(8.0, 54.0, 984.0, 492.0),
    seen: true,
    ..Pane::default()
  })];
  state.workspace_transform = transform;
  state
}

/// Where the panes are on the screen, in points.
fn on_screen(state: &SurfaceState) -> [f64; 4] {
  let shown = displayed(state).expect("a pane is shown");
  let origin = state.window_follow.origin.expect("the corner is known");
  rounded(rect(
    f64::from(origin.0) / state.scale + state.viewport.x + shown.x,
    f64::from(origin.1) / state.scale + state.viewport.y + shown.y,
    shown.width,
    shown.height,
  ))
}

/// The window grown 100 points at its left edge and 50 at its bottom.
fn grow_left_and_down(state: &mut SurfaceState) {
  set_origin(state, Some((0, 200)));
  set_page_viewport(state, rect(0.0, 40.0, 1100.0, 650.0), Some((1300.0, 850.0)));
}

#[test]
fn a_zoomed_picture_stays_where_it_is_on_screen_as_the_window_is_resized() {
  let mut state = laid_out(WorkspaceTransform {
    pan_x: 30.0,
    pan_y: -20.0,
    zoom: 2.0,
  });
  let before = on_screen(&state);

  grow_left_and_down(&mut state);

  assert_eq!(on_screen(&state), before);
}

#[test]
fn a_fitted_picture_is_fitted_into_the_resized_window() {
  let mut state = laid_out(WorkspaceTransform::default());
  let before = displayed(&state).expect("a pane is shown");

  grow_left_and_down(&mut state);

  assert!(is_fitted(state.workspace_transform));
  assert!(displayed(&state).expect("a pane is shown").width > before.width);
}
