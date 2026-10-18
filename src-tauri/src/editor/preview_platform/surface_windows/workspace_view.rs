// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The workspace drawable: one swap chain over the whole viewport that the
//! shared scene draws every layer into, over its checkerboard, as the macOS
//! workspace layer is drawn.

use super::*;
use screenwide_osc::gpu::magnifier::NativeMagnifier;
use screenwide_workspace::backdrop::BackdropStyle;
use screenwide_workspace::scene::{NativeWorkspacePlacement, WorkspaceScene};

pub(super) struct WorkspaceView {
  pub(super) scene: WorkspaceScene,
  gpu: &'static screenwide_gpu::Gpu,
  drawable: Mutex<Drawable>,
  visual: IDCompositionVisual,
}

/// The swap chain and where its visual was last put, in physical pixels;
/// `None` while it is off screen.
struct Drawable {
  surface: Surface,
  offset: Option<(i32, i32)>,
}

impl WorkspaceView {
  pub(super) fn new(
    gpu: &'static screenwide_gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
    above: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows workspace visual could not be created: {error}"))?;
    unsafe {
      visual
        .SetOffsetX2(-100_000.0)
        .and_then(|_| root.AddVisual(&visual, true, Some(above)))
    }
    .map_err(|error| format!("The Windows workspace visual could not be attached: {error}"))?;
    let mut surface = Surface::on_visual(gpu, &visual)?;
    // The scene composes into a picture of its own and copies it in, so a
    // redraw of an unchanged scene costs one copy.
    surface.accept_copies(gpu);
    Ok(Self {
      scene: WorkspaceScene::new()?,
      gpu,
      drawable: Mutex::new(Drawable {
        surface,
        offset: None,
      }),
      visual,
    })
  }

  /// Takes the drawable off screen and gives its memory back. Answers whether
  /// it was on screen, which the caller has to commit.
  pub(super) fn hide(&self) -> bool {
    let Ok(mut drawable) = self.drawable.lock() else {
      return false;
    };
    if drawable.offset.take().is_none() {
      return false;
    }
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
    drawable.surface.resize(self.gpu, (2, 2));
    true
  }

  /// Draws the scene into the drawable laid over `frame`, x, y, width and
  /// height in physical pixels. Answers whether the visual moved, which the
  /// caller has to commit.
  pub(super) fn draw(
    &self,
    frame: (i32, i32, u32, u32),
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
  ) -> Result<bool, String> {
    let mut drawable = self
      .drawable
      .lock()
      .map_err(|_| "The Windows workspace drawable is poisoned".to_owned())?;
    let (x, y, width, height) = frame;
    drawable.surface.resize(self.gpu, (width, height));
    let moved = drawable.offset != Some((x, y));
    if moved {
      unsafe {
        self
          .visual
          .SetOffsetX2(x as f32)
          .and_then(|_| self.visual.SetOffsetY2(y as f32))
      }
      .map_err(|error| error.to_string())?;
      drawable.offset = Some((x, y));
    }
    // Nothing can be drawn now; the next change draws the scene again.
    let Frame::Ready(target) = drawable.surface.acquire(self.gpu)? else {
      return Ok(moved);
    };
    self
      .scene
      .draw(&target.texture, placements, magnifier, backdrop, true)?;
    self.gpu.queue.present(target);
    Ok(moved)
  }
}
