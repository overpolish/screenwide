// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rounded corners the picture meets the window's chrome on: one
//! rectangle clip over the visuals under the webview - the backstop, the
//! workspace and the ribbon, every one laid over the viewport - and one over
//! the chrome on the input window. The twin of `Layers::round` on macOS.

use screenwide_compositor::PreviewSurfaceCorners;
use windows::Win32::Graphics::DirectComposition::{
  IDCompositionDevice, IDCompositionRectangleClip, IDCompositionVisual,
};

pub(super) struct ViewportClip {
  picture: IDCompositionRectangleClip,
  chrome: IDCompositionRectangleClip,
}

impl ViewportClip {
  /// Clips `root`, in the host window's pixels, and `editor_root`, in the
  /// input window's. The caller commits the tree.
  pub(super) fn new(
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
    editor_root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    (|| -> windows::core::Result<Self> {
      let picture = unsafe { composition.CreateRectangleClip() }?;
      let chrome = unsafe { composition.CreateRectangleClip() }?;
      unsafe {
        root.SetClip(&picture)?;
        editor_root.SetClip(&chrome)?;
      }
      Ok(Self { picture, chrome })
    })()
    .map_err(|error| format!("The Windows preview clip could not be created: {error}"))
  }

  /// Clips to the viewport over `frame`, x, y, width and height in physical
  /// pixels from the host window's content corner, rounded at `corners`, in
  /// logical points at `scale`. The caller commits.
  pub(super) fn set(
    &self,
    (x, y, width, height): (i32, i32, i32, i32),
    corners: PreviewSurfaceCorners,
    scale: f64,
  ) {
    let (x, y, width, height) = (x as f32, y as f32, width as f32, height as f32);
    let _ = shape(&self.picture, (x, y, x + width, y + height), corners, scale);
    let _ = shape(&self.chrome, (0.0, 0.0, width, height), corners, scale);
  }
}

fn shape(
  clip: &IDCompositionRectangleClip,
  (left, top, right, bottom): (f32, f32, f32, f32),
  corners: PreviewSurfaceCorners,
  scale: f64,
) -> windows::core::Result<()> {
  let pixels = |points: f64| (points.max(0.0) * scale) as f32;
  let (top_left, top_right) = (pixels(corners.top_left), pixels(corners.top_right));
  let (bottom_right, bottom_left) = (pixels(corners.bottom_right), pixels(corners.bottom_left));
  unsafe {
    clip.SetLeft2(left)?;
    clip.SetTop2(top)?;
    clip.SetRight2(right)?;
    clip.SetBottom2(bottom)?;
    clip.SetTopLeftRadiusX2(top_left)?;
    clip.SetTopLeftRadiusY2(top_left)?;
    clip.SetTopRightRadiusX2(top_right)?;
    clip.SetTopRightRadiusY2(top_right)?;
    clip.SetBottomRightRadiusX2(bottom_right)?;
    clip.SetBottomRightRadiusY2(bottom_right)?;
    clip.SetBottomLeftRadiusX2(bottom_left)?;
    clip.SetBottomLeftRadiusY2(bottom_left)
  }
}
