// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral geometry for treating every preview pane as one workspace.

use super::PreviewSurfaceRect;

#[derive(Clone, Copy)]
#[repr(C)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(super) struct WorkspaceTransform {
  pub pan_x: f64,
  pub pan_y: f64,
  pub zoom: f64,
}

impl Default for WorkspaceTransform {
  fn default() -> Self {
    Self {
      pan_x: 0.0,
      pan_y: 0.0,
      zoom: 1.0,
    }
  }
}

impl WorkspaceTransform {
  /// Reset to 100% and centre the workspace before the panel once.
  /// A constrained window allows overlap instead of reducing zoom.
  pub(super) fn for_panel(
    base: PreviewSurfaceRect,
    viewport: PreviewSurfaceRect,
    requested_width: f64,
  ) -> Self {
    if !requested_width.is_finite()
      || requested_width <= 0.0
      || ![base.x, base.y, base.width, base.height]
        .iter()
        .all(|v| v.is_finite())
      || !viewport.width.is_finite()
      || !viewport.height.is_finite()
      || base.width <= 0.0
      || base.height <= 0.0
      || viewport.width <= 0.0
      || viewport.height <= 0.0
    {
      return Self::default();
    }
    let usable_width = requested_width.clamp(0.0, viewport.width);
    let base_center_x = base.x + base.width / 2.0;
    let base_center_y = base.y + base.height / 2.0;
    Self {
      pan_x: usable_width / 2.0 - base_center_x,
      pan_y: viewport.height / 2.0 - base_center_y,
      zoom: 1.0,
    }
  }

  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(super) fn apply(
    self,
    viewport: PreviewSurfaceRect,
    pane: PreviewSurfaceRect,
  ) -> PreviewSurfaceRect {
    let viewport_center_x = viewport.width / 2.0;
    let viewport_center_y = viewport.height / 2.0;
    let pane_center_x = pane.x + pane.width / 2.0;
    let pane_center_y = pane.y + pane.height / 2.0;
    let width = pane.width * self.zoom;
    let height = pane.height * self.zoom;
    let center_x = viewport_center_x + self.pan_x + (pane_center_x - viewport_center_x) * self.zoom;
    let center_y = viewport_center_y + self.pan_y + (pane_center_y - viewport_center_y) * self.zoom;
    PreviewSurfaceRect {
      height,
      width,
      x: center_x - width / 2.0,
      y: center_y - height / 2.0,
    }
  }
}

/// C ABI used by the macOS surface; Windows calls the same calculation directly.
#[no_mangle]
pub(super) extern "C" fn screenwide_workspace_panel_fit(
  viewport_width: f64,
  viewport_height: f64,
  base_x: f64,
  base_y: f64,
  base_width: f64,
  base_height: f64,
  fit_width: f64,
) -> WorkspaceTransform {
  WorkspaceTransform::for_panel(
    PreviewSurfaceRect {
      x: base_x,
      y: base_y,
      width: base_width,
      height: base_height,
    },
    PreviewSurfaceRect {
      x: 0.0,
      y: 0.0,
      width: viewport_width,
      height: viewport_height,
    },
    fit_width,
  )
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

  #[test]
  fn multiple_panes_keep_their_relative_workspace_geometry() {
    let viewport = rect(100.0, 50.0, 1_000.0, 600.0);
    let transform = WorkspaceTransform {
      pan_x: 25.0,
      pan_y: -10.0,
      zoom: 2.0,
    };
    let screen = transform.apply(viewport, rect(100.0, 150.0, 300.0, 200.0));
    let camera = transform.apply(viewport, rect(424.0, 175.0, 200.0, 150.0));

    assert_eq!(
      (screen.x, screen.y, screen.width, screen.height),
      (-275.0, -10.0, 600.0, 400.0)
    );
    assert_eq!(
      (camera.x, camera.y, camera.width, camera.height),
      (373.0, 40.0, 400.0, 300.0)
    );
    assert_eq!(camera.x - (screen.x + screen.width), 48.0);
  }

  #[test]
  fn identity_leaves_each_pane_unchanged() {
    let pane = rect(27.0, 31.0, 640.0, 360.0);
    let transformed = WorkspaceTransform::default().apply(rect(0.0, 0.0, 900.0, 600.0), pane);
    assert_eq!(
      (
        transformed.x,
        transformed.y,
        transformed.width,
        transformed.height
      ),
      (pane.x, pane.y, pane.width, pane.height)
    );
  }

  #[test]
  fn panel_fit_uses_requested_width_and_centres_in_reserved_area() {
    let transform = WorkspaceTransform::for_panel(
      rect(100.0, 50.0, 700.0, 400.0),
      rect(0.0, 0.0, 1_000.0, 600.0),
      800.0,
    );
    assert_eq!(transform.zoom, 1.0);
    assert_eq!(transform.pan_x, -50.0);
    assert_eq!(transform.pan_y, 50.0);
  }

  #[test]
  fn panel_reset_keeps_100_percent_when_the_window_narrows() {
    let transform = WorkspaceTransform::for_panel(
      rect(0.0, 0.0, 900.0, 400.0),
      rect(0.0, 0.0, 1_000.0, 600.0),
      900.0,
    );
    let narrowed = transform.apply(rect(0.0, 0.0, 500.0, 600.0), rect(0.0, 0.0, 900.0, 400.0));
    assert_eq!(transform.zoom, 1.0);
    assert!(narrowed.width > 500.0);
  }

  #[test]
  fn activating_after_different_window_sizes_always_uses_100_percent() {
    for (width, height, available) in [
      (1200.0, 800.0, 880.0),
      (900.0, 500.0, 580.0),
      (500.0, 200.0, 180.0),
    ] {
      let viewport = rect(0.0, 0.0, width, height);
      let base = rect(8.0, 8.0, width - 16.0, height - 16.0);
      let transform = WorkspaceTransform::for_panel(base, viewport, available);
      let shown = transform.apply(viewport, base);
      assert_eq!(transform.zoom, 1.0);
      assert_eq!((shown.width, shown.height), (base.width, base.height));
      assert_eq!(shown.x + shown.width / 2.0, available / 2.0);
    }
  }

  #[test]
  fn default_reset_discards_panel_transform_and_centres_full_frame() {
    let viewport = rect(0.0, 0.0, 1000.0, 600.0);
    let base = rect(50.0, 50.0, 900.0, 500.0);
    let panel = WorkspaceTransform::for_panel(base, viewport, 680.0).apply(viewport, base);
    assert_eq!(panel.width, base.width);
    assert_eq!(panel.x + panel.width / 2.0, 340.0);
    assert!(panel.x + panel.width > 680.0);
    let reset = screenwide_workspace_panel_fit(1000.0, 600.0, 50.0, 50.0, 900.0, 500.0, 0.0);
    let shown = reset.apply(viewport, base);
    assert_eq!((shown.x, shown.width, reset.zoom), (50.0, 900.0, 1.0));
  }

  /// The double-click reset applies the stored basis: a panel width while a
  /// tool panel holds one, zero once it hands the basis back.
  #[test]
  fn a_reset_follows_the_stored_fit_basis() {
    let viewport = rect(0.0, 0.0, 1_000.0, 600.0);
    let base = rect(50.0, 50.0, 900.0, 500.0);
    let opened = WorkspaceTransform::for_panel(base, viewport, 680.0);
    let reset_again = WorkspaceTransform::for_panel(base, viewport, 680.0);
    assert_eq!(
      (reset_again.pan_x, reset_again.pan_y, reset_again.zoom),
      (opened.pan_x, opened.pan_y, opened.zoom)
    );
    assert_eq!(opened.apply(viewport, base).x + base.width / 2.0, 340.0);

    let cleared = WorkspaceTransform::for_panel(base, viewport, 0.0);
    assert_eq!(
      (cleared.pan_x, cleared.pan_y, cleared.zoom),
      (0.0, 0.0, 1.0)
    );
  }

  #[test]
  fn invalid_fit_inputs_are_identity_and_do_not_panic() {
    for width in [0.0, -1.0, f64::NAN, f64::INFINITY] {
      let fit = screenwide_workspace_panel_fit(1000.0, 600.0, 0.0, 0.0, 900.0, 500.0, width);
      assert_eq!((fit.zoom, fit.pan_x, fit.pan_y), (1.0, 0.0, 0.0));
    }
    let fit = screenwide_workspace_panel_fit(-1.0, 600.0, 0.0, 0.0, 900.0, 500.0, 680.0);
    assert_eq!(fit.zoom, 1.0);
  }

  #[test]
  fn invalid_base_or_viewport_returns_default() {
    assert_eq!(
      WorkspaceTransform::for_panel(
        rect(0.0, 0.0, 0.0, 400.0),
        rect(0.0, 0.0, 500.0, 600.0),
        300.0
      )
      .zoom,
      1.0
    );
  }
}
