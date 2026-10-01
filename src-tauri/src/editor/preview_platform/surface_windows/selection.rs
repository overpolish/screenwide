// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transparent selection overlay composed above the preview panes, drawn
//! through the shared OSC shader.

#[path = "selection/drawing.rs"]
mod drawing;

use windows::Win32::Graphics::DirectComposition::{IDCompositionDevice, IDCompositionVisual};

use crate::osc::{
  geometry::{Point, Rect, Size},
  gpu::windows::{self as osc_gpu, RenderConstants, Vertex},
};
use crate::windows::overlay_surface::{Frame, VisualSurface};

fn logical_rect(rect: [f32; 4], scale: f64) -> Rect {
  Rect::from_xywh(
    f64::from(rect[0]) / scale,
    f64::from(rect[1]) / scale,
    f64::from(rect[2]) / scale,
    f64::from(rect[3]) / scale,
  )
}

pub(super) struct SelectionOverlay {
  shared: &'static crate::gpu::Gpu,
  surface: VisualSurface,
  pipeline: wgpu::RenderPipeline,
  /// The constants block, and transparent texels for every texture the
  /// shader declares, which the overlay's own quads never sample.
  bindings: wgpu::BindGroup,
  constants: wgpu::Buffer,
  vertex_buffer: wgpu::Buffer,
  vertex_capacity: usize,
  /// Held only to keep the composition visual alive: the surface is attached
  /// once and no property is mutated after construction.
  _visual: IDCompositionVisual,
}

impl SelectionOverlay {
  /// Gives the surface's memory back while the editor is hidden.
  pub(super) fn release_drawables(&mut self) {
    self.surface.resize(self.shared, (2, 2));
  }

  /// The overlay's visual under `root`. The caller commits the tree.
  pub(super) fn new(
    shared: &'static crate::gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe { root.AddVisual(&visual, true, None::<&IDCompositionVisual>) }
      .map_err(|error| error.to_string())?;
    let surface = VisualSurface::new(shared, &visual)?;
    let device = &shared.device;
    let module = osc_gpu::shader_module(device);
    let layout = osc_gpu::bind_group_layout(device);
    let pipeline = osc_gpu::pipeline(device, &layout, &module, wgpu::BlendFactor::SrcAlpha);
    let constants = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide selection constants"),
      size: size_of::<RenderConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let placeholder = shared
      .texture_with_pixels(
        "Screenwide selection placeholder",
        (1, 1, 1),
        wgpu::TextureFormat::Bgra8Unorm,
        &[0; 4],
      )
      .create_view(&Default::default());
    let texture = |binding| wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::TextureView(&placeholder),
    };
    let linear_sampler = osc_gpu::sampler(device, wgpu::FilterMode::Linear);
    let point_sampler = osc_gpu::sampler(device, wgpu::FilterMode::Nearest);
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide selection bindings"),
      layout: &layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &constants,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<RenderConstants>() as u64),
          }),
        },
        texture(1),
        texture(2),
        texture(3),
        texture(4),
        texture(5),
        wgpu::BindGroupEntry {
          binding: 6,
          resource: wgpu::BindingResource::Sampler(&linear_sampler),
        },
        wgpu::BindGroupEntry {
          binding: 7,
          resource: wgpu::BindingResource::Sampler(&point_sampler),
        },
      ],
    });
    let vertex_capacity = 256;
    Ok(Self {
      shared,
      surface,
      pipeline,
      bindings,
      constants,
      vertex_buffer: vertex_buffer(shared, vertex_capacity),
      vertex_capacity,
      _visual: visual,
    })
  }
}

fn vertex_buffer(gpu: &crate::gpu::Gpu, capacity: usize) -> wgpu::Buffer {
  gpu.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Screenwide selection vertices"),
    size: (capacity * size_of::<Vertex>()) as u64,
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  })
}
