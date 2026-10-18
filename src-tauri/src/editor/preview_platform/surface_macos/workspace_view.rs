// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The workspace drawable: one Metal layer over the whole viewport that the
//! shared scene draws every layer into, over its checkerboard. The twin of
//! the Windows workspace visual.

use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2_quartz_core::CAMetalLayer;
use screenwide_osc::gpu::magnifier::NativeMagnifier;
use screenwide_workspace::backdrop::BackdropStyle;
use screenwide_workspace::scene::{NativeWorkspacePlacement, WorkspaceScene};

use super::layers::place;
use super::*;

pub(super) struct WorkspaceView {
  pub(super) scene: WorkspaceScene,
  gpu: &'static screenwide_gpu::Gpu,
  drawable: Mutex<Drawable>,
  layer: Retained<CAMetalLayer>,
}

// The layer is only moved inside transactions, which Core Animation allows
// from any thread, and drawn through wgpu.
unsafe impl Send for WorkspaceView {}
unsafe impl Sync for WorkspaceView {}

/// The drawable and where its layer was last put, in physical pixels;
/// `None` while it is off screen.
struct Drawable {
  surface: Surface,
  frame: Option<(i32, i32, u32, u32)>,
}

/// A wgpu surface on `layer`.
///
/// # Safety
/// Main thread only, as wgpu configures the layer as it is made.
pub(super) unsafe fn surface_on(
  gpu: &'static screenwide_gpu::Gpu,
  layer: &CAMetalLayer,
) -> Result<Surface, String> {
  let pointer = NonNull::from(layer).cast::<std::ffi::c_void>();
  unsafe { Surface::on_metal_layer(gpu, pointer) }
}

impl WorkspaceView {
  /// # Safety
  /// Main thread only.
  pub(super) unsafe fn new(
    gpu: &'static screenwide_gpu::Gpu,
    layer: Retained<CAMetalLayer>,
  ) -> Result<Self, String> {
    let mut surface = unsafe { surface_on(gpu, &layer) }?;
    // The scene composes into a picture of its own and copies it in, so a
    // redraw of an unchanged scene costs one copy.
    surface.accept_copies(gpu);
    Ok(Self {
      scene: WorkspaceScene::new()?,
      gpu,
      drawable: Mutex::new(Drawable {
        surface,
        frame: None,
      }),
      layer,
    })
  }

  /// Takes the drawable off screen and gives its memory back. Answers whether
  /// it was on screen.
  pub(super) fn hide(&self) -> bool {
    let Ok(mut drawable) = self.drawable.lock() else {
      return false;
    };
    if drawable.frame.take().is_none() {
      return false;
    }
    place(&self.layer, None);
    drawable.surface.resize(self.gpu, (2, 2));
    true
  }

  /// Draws the scene into the drawable laid over `frame`, x, y, width and
  /// height in physical pixels, at `scale` pixels to the point. Answers
  /// whether the layer moved.
  pub(super) fn draw(
    &self,
    frame: (i32, i32, u32, u32),
    scale: f64,
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
  ) -> Result<bool, String> {
    let mut drawable = self
      .drawable
      .lock()
      .map_err(|_| "The macOS workspace drawable is poisoned".to_owned())?;
    let (x, y, width, height) = frame;
    drawable.surface.resize(self.gpu, (width, height));
    let moved = drawable.frame != Some(frame);
    if moved {
      let scale = scale.max(0.1);
      place(
        &self.layer,
        Some((
          f64::from(x) / scale,
          f64::from(y) / scale,
          f64::from(width) / scale,
          f64::from(height) / scale,
        )),
      );
      drawable.frame = Some(frame);
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
