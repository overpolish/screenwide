// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A short-video app's interface, and the phone it is previewed on, drawn
//! over the canvas in the chrome drawable, under the selection chrome. The
//! chrome is never composed into a picture, so none of it reaches an export.

use std::sync::Arc;

use screenwide_gpu::{Gpu, RenderStages, Shader};

use crate::editing::overlay_raster::RasteredOverlay;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/platform_overlay.wgsl"));

/// One overlay drawn to pixels, the canvas it is laid over, in the chrome
/// drawable's pixels (x, y, width, height), and how opaque it is drawn while
/// it fades in or out. The picture may have been drawn for another size of
/// the canvas, and is scaled to this one.
#[derive(Clone)]
pub struct ChromeOverlay {
  pub picture: Arc<RasteredOverlay>,
  pub rect: [f32; 4],
  pub opacity: f32,
}

/// The twin of `Placement` in `platform_overlay.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Placement {
  rect: [f32; 4],
  target_size: [f32; 4],
}

/// A picture on the GPU with its own placement, kept while it is drawn.
struct Drawn {
  picture: Arc<RasteredOverlay>,
  placement: wgpu::Buffer,
  bindings: wgpu::BindGroup,
}

pub(super) struct OverlayPainter {
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The overlays drawn now, back to front: one, or two while one fades
  /// into the other.
  drawn: Vec<Drawn>,
}

impl OverlayPainter {
  pub(super) fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide platform overlay",
      &crate::shader_layouts::PLATFORM_OVERLAY,
      SHADER,
      crate::precompiled_shaders::PLATFORM_OVERLAY,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide platform overlay",
      vertex: "vs_main",
      fragment: "fs_main",
      targets: &[Some(wgpu::ColorTargetState {
        format: screenwide_gpu::surface::FORMAT,
        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Screenwide platform overlay sampler"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });
    Self {
      pipeline,
      layout: shader.bind_group_layout(0).clone(),
      sampler,
      drawn: Vec::new(),
    }
  }

  /// Gets `overlays` ready to draw, back to front, on a drawable `size`
  /// pixels: a picture sent to the GPU only the first time it is drawn, and
  /// each placed over the canvas at its opacity.
  pub(super) fn prepare(&mut self, gpu: &Gpu, overlays: &[ChromeOverlay], size: (u32, u32)) {
    let mut kept = std::mem::take(&mut self.drawn);
    for overlay in overlays {
      let drawn = match kept
        .iter()
        .position(|drawn| Arc::ptr_eq(&drawn.picture, &overlay.picture))
      {
        Some(index) => kept.swap_remove(index),
        None => self.upload(gpu, &overlay.picture),
      };
      // A phone's bars reach past the canvas, so the picture is placed from
      // the canvas's corner by its own share of the canvas.
      let place = overlay.picture.place;
      let [x, y, width, height] = overlay.rect;
      gpu.queue.write_buffer(
        &drawn.placement,
        0,
        bytemuck::bytes_of(&Placement {
          rect: [
            x + place[0] * width,
            y + place[1] * height,
            place[2] * width,
            place[3] * height,
          ],
          target_size: [size.0 as f32, size.1 as f32, overlay.opacity, 0.0],
        }),
      );
      self.drawn.push(drawn);
    }
  }

  fn upload(&self, gpu: &Gpu, picture: &Arc<RasteredOverlay>) -> Drawn {
    let pixels = &picture.picture;
    let texture = gpu.texture_with_pixels(
      "Screenwide platform overlay",
      (pixels.size.0, pixels.size.1, 1),
      wgpu::TextureFormat::Rgba8Unorm,
      &pixels.pixels,
    );
    let view = texture.create_view(&Default::default());
    let placement = gpu.device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide platform overlay placement"),
      size: size_of::<Placement>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide platform overlay bindings"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: placement.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&view),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    });
    Drawn {
      picture: Arc::clone(picture),
      placement,
      bindings,
    }
  }

  /// Draws what [`Self::prepare`] made ready.
  pub(super) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
    pass.set_pipeline(&self.pipeline);
    for drawn in &self.drawn {
      pass.set_bind_group(0, &drawn.bindings, &[]);
      pass.draw(0..6, 0..1);
    }
  }
}
