// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The backstop below the workspace drawable: the webview's backdrop colour
//! over the whole preview viewport, the native equivalent of macOS's
//! container layer. The workspace scene draws its checkerboard over it.

use super::*;

pub(super) struct Backdrop {
  gpu: &'static screenwide_gpu::Gpu,
  scale_transform: IDCompositionScaleTransform,
  surface: Surface,
  pub(super) visual: IDCompositionVisual,
}

impl Backdrop {
  pub(super) fn new(
    gpu: &'static screenwide_gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| {
      format!("The Windows preview backstop visual could not be created: {error}")
    })?;
    let scale_transform = unsafe { composition.CreateScaleTransform() }.map_err(|error| {
      format!("The Windows preview backstop transform could not be created: {error}")
    })?;
    (|| -> windows::core::Result<()> {
      unsafe {
        visual.SetTransform(&scale_transform)?;
        visual.SetOffsetX2(-100_000.0)?;
        root.AddVisual(&visual, false, None::<&IDCompositionVisual>)?;
      }
      Ok(())
    })()
    .map_err(|error| format!("The Windows preview backstop could not be attached: {error}"))?;
    // Two pixels square, stretched over the viewport by the scale transform.
    let surface = Surface::on_visual(gpu, &visual)?;
    let backdrop = Self {
      gpu,
      scale_transform,
      surface,
      visual,
    };
    backdrop.paint([0.09, 0.09, 0.10, 1.0])?;
    Ok(backdrop)
  }

  /// Fills the backstop with `colour`, the webview's backdrop, straight alpha.
  pub(super) fn paint(&self, colour: [f64; 4]) -> Result<(), String> {
    let Frame::Ready(frame) = self.surface.acquire(self.gpu)? else {
      return Err("The Windows preview backstop has no frame to paint".to_owned());
    };
    let alpha = colour[3].clamp(0.0, 1.0);
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = self
      .gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide preview backstop"),
      });
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Screenwide preview backstop"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color {
            r: colour[0].clamp(0.0, 1.0) * alpha,
            g: colour[1].clamp(0.0, 1.0) * alpha,
            b: colour[2].clamp(0.0, 1.0) * alpha,
            a: alpha,
          }),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    self.gpu.queue.submit([encoder.finish()]);
    self.gpu.queue.present(frame);
    Ok(())
  }

  pub(super) fn set_geometry(&self, rect: PreviewSurfaceRect, scale: f64) {
    if rect.width < 1.0 || rect.height < 1.0 {
      self.hide();
      return;
    }
    let (x, right) = pixels::scaled_edges(rect.x, rect.width, scale);
    let (y, bottom) = pixels::scaled_edges(rect.y, rect.height, scale);
    let _ = unsafe {
      self
        .visual
        .SetOffsetX2(x as f32)
        .and_then(|_| self.visual.SetOffsetY2(y as f32))
        .and_then(|_| {
          self
            .scale_transform
            .SetScaleX2((right - x).max(2) as f32 / 2.0)
        })
        .and_then(|_| {
          self
            .scale_transform
            .SetScaleY2((bottom - y).max(2) as f32 / 2.0)
        })
    };
  }

  pub(super) fn hide(&self) {
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
  }
}
