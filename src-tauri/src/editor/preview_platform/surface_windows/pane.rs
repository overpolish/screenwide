// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Backdrop {
  pub(super) fn new(
    gpu: &crate::gpu::Gpu,
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
        // This is the native equivalent of macOS's opaque container layer: it
        // sits below every video pane but fills the complete preview viewport.
        root.AddVisual(&visual, false, None::<&IDCompositionVisual>)?;
      }
      Ok(())
    })()
    .map_err(|error| format!("The Windows preview backstop could not be attached: {error}"))?;
    // Two pixels square, stretched over the viewport by the scale transform.
    let surface = VisualSurface::new(gpu, &visual)?;
    Ok(Self {
      scale_transform,
      surface,
      visual,
    })
  }

  pub(super) fn paint(&self, gpu: &crate::gpu::Gpu, colour: [f64; 4]) -> Result<(), String> {
    let Frame::Ready(frame) = self.surface.acquire(gpu)? else {
      return Err("The Windows preview backstop has no frame to paint".to_owned());
    };
    let alpha = colour[3].clamp(0.0, 1.0);
    clear(
      gpu,
      &frame.texture,
      wgpu::Color {
        r: colour[0].clamp(0.0, 1.0) * alpha,
        g: colour[1].clamp(0.0, 1.0) * alpha,
        b: colour[2].clamp(0.0, 1.0) * alpha,
        a: alpha,
      },
    );
    gpu.queue.present(frame);
    Ok(())
  }

  pub(super) fn set_geometry(&self, rect: PreviewSurfaceRect, scale: f64) {
    if rect.width < 1.0 || rect.height < 1.0 {
      self.hide();
      return;
    }
    let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
    let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
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

impl Pane {
  /// Gives the surface's memory back while the pane is hidden. A parked
  /// frame is dropped unshown first: no frame may be held across a resize.
  pub(super) fn release_drawables(&mut self, gpu: &crate::gpu::Gpu) {
    self.parked = None;
    self.surface.resize(gpu, (2, 2));
  }

  pub(super) fn update_geometry(&self) -> windows::core::Result<()> {
    let content_width = self.content_size.0.max(1) as f32;
    let content_height = self.content_size.1.max(1) as f32;
    let display_width = self.display_size.0.max(1) as f32;
    let display_height = self.display_size.1.max(1) as f32;
    let (clip_left, clip_top, clip_right, clip_bottom) = self.clip_edges;
    // One uniform scale, so a composed canvas whose aspect no longer matches
    // the laid-out box is centred inside it rather than stretched across it.
    // Aspects agree in the steady state and this is then exactly the
    // per-axis mapping; it only bites in the window between a re-composed
    // canvas and the layout that catches up with it.
    let scale = (display_width / content_width).min(display_height / content_height);
    let inset_x = (display_width - content_width * scale) / 2.0;
    let inset_y = (display_height - content_height * scale) / 2.0;
    unsafe {
      self.visual.SetOffsetX2(self.position.0 as f32 + inset_x)?;
      self.visual.SetOffsetY2(self.position.1 as f32 + inset_y)?;
      self.scale_transform.SetScaleX2(scale)?;
      self.scale_transform.SetScaleY2(scale)?;
      self.clip.SetLeft2((clip_left as f32 - inset_x) / scale)?;
      self.clip.SetTop2((clip_top as f32 - inset_y) / scale)?;
      self.clip.SetRight2((clip_right as f32 - inset_x) / scale)?;
      self
        .clip
        .SetBottom2((clip_bottom as f32 - inset_y) / scale)?;
    }
    Ok(())
  }

  pub(super) fn hide(&self) {
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
  }
}

/// Fills `texture` with `colour` and submits the clear.
pub(super) fn clear(gpu: &crate::gpu::Gpu, texture: &wgpu::Texture, colour: wgpu::Color) {
  let view = texture.create_view(&Default::default());
  let mut encoder = gpu
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Screenwide preview clear"),
    });
  encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide preview clear"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view: &view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(colour),
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  gpu.queue.submit([encoder.finish()]);
}
