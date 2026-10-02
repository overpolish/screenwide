// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's pipeline: one shader on the shared device, one buffer per
//! list.

use crate::editor::preview_platform::annotation_gpu::{
  numbered_arrows, CounterAtlas, GpuBuffer, PreparedArrows,
};
use crate::gpu::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/annotate_overlay.wgsl"));

/// The twin of `Overlay` in `annotate_overlay.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Constants {
  count: u32,
  /// Half the width an edge is smoothed over, as `composite_annotations`
  /// takes it. The overlay draws at the display's own resolution, so the edge
  /// spans one layer pixel and this is half of one.
  feather: f32,
  /// The size of the texture the counters' numbers were rasterised into, in
  /// pixels, or zeroes when no counter is on screen to rasterise one.
  atlas: [u32; 2],
  /// How many atlas pixels that texture holds per layer pixel.
  atlas_scale: f32,
  /// The target's size in pixels, which the highlights' underlay is read
  /// across.
  target: [f32; 2],
  spare: f32,
}

const _: () = assert!(std::mem::size_of::<Constants>() == 32);

/// Everything every surface of one session shares.
pub(in crate::annotate) struct Renderer {
  gpu: &'static Gpu,
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  constants: wgpu::Buffer,
  arrows: GpuBuffer,
  samples: GpuBuffer,
  /// A highlight's bands.
  points: GpuBuffer,
  text: GpuBuffer,
  /// The counters' numbers, rasterised at the size they are drawn into an
  /// atlas that keeps them, so a frame that redraws an unchanged screen does
  /// no work.
  counters: CounterAtlas,
  /// Bound where the numbers, the underlay or its softened copy go when there
  /// is none: one transparent pixel, which the shader reads as none.
  empty: wgpu::TextureView,
}

impl Renderer {
  pub(in crate::annotate) fn new() -> Result<Self, String> {
    let gpu = crate::gpu::shared()?;
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide annotate overlay shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = bind_group_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide annotate overlay layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    // The result is already premultiplied, so it is written rather than
    // blended: one pass over a cleared target has nothing to blend with.
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Screenwide annotate overlay pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: crate::gpu::surface::FORMAT,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    });
    Ok(Self {
      gpu,
      pipeline,
      layout,
      constants: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenwide annotate overlay constants"),
        size: size_of::<Constants>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      arrows: GpuBuffer::new(gpu, "Screenwide annotate arrows"),
      samples: GpuBuffer::new(gpu, "Screenwide annotate exposure"),
      points: GpuBuffer::new(gpu, "Screenwide annotate highlight bands"),
      text: GpuBuffer::new(gpu, "Screenwide annotate text"),
      counters: CounterAtlas::default(),
      empty: gpu
        .texture_with_pixels(
          "Screenwide annotate placeholder",
          (1, 1, 1),
          wgpu::TextureFormat::Rgba8Unorm,
          &[0; 4],
        )
        .create_view(&Default::default()),
    })
  }

  pub(in crate::annotate) fn gpu(&self) -> &'static Gpu {
    self.gpu
  }

  /// Draws the prepared annotations over a cleared target of `size` physical
  /// pixels. Shared by a display's frame and by a still that leaves the app as
  /// pixels, so both draw the same annotations the same way.
  ///
  /// A counter's number is type rather than a shape the shader can solve, so
  /// it is rasterised here, at the size it is drawn, and handed over as one
  /// texture - the same pass the editor's compositor makes. `underlay` is what
  /// a highlight recolours: the desktop captured under it, or the still it is
  /// baked into. Without one there is nothing under a highlight to read.
  /// `softened` is the same softened for a spotlight's blur; without one a
  /// spotlight only shades.
  pub(in crate::annotate) fn draw_arrows(
    &self,
    target: &wgpu::TextureView,
    size: (u32, u32),
    prepared: &PreparedArrows,
    underlay: Option<&wgpu::TextureView>,
    softened: Option<&wgpu::TextureView>,
  ) -> Result<(), String> {
    let gpu = self.gpu;
    let (numbers, numbered) = numbered_arrows(&self.counters, gpu, prepared)?;
    let constants = Constants {
      count: numbered.len() as u32,
      feather: 0.5,
      atlas: numbers.as_ref().map_or([0, 0], |atlas| {
        let (width, height) = atlas.size;
        [width, height]
      }),
      atlas_scale: numbers.as_ref().map_or(0.0, |atlas| atlas.scale),
      target: [size.0 as f32, size.1 as f32],
      spare: 0.0,
    };
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    let arrow_buffer = self.arrows.write(gpu, &numbered)?;
    let sample_buffer = self.samples.write(gpu, &prepared.samples)?;
    let point_buffer = self.points.write(gpu, &prepared.points)?;
    let text_buffer = self.text.write(gpu, &prepared.text)?;
    fn texture(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
      wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
      }
    }
    fn buffer(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
      wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
      }
    }
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide annotate overlay bindings"),
      layout: &self.layout,
      entries: &[
        buffer(0, &self.constants),
        texture(1, underlay.unwrap_or(&self.empty)),
        texture(2, softened.unwrap_or(&self.empty)),
        buffer(7, &arrow_buffer),
        buffer(8, &sample_buffer),
        texture(9, numbers.as_ref().map_or(&self.empty, |atlas| &atlas.view)),
        buffer(10, &point_buffer),
        buffer(11, &text_buffer),
      ],
    });
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide annotate overlay"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide annotate overlay"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    Ok(())
  }
}

/// The overlay's own uniform and pictures, then the annotation lists and
/// atlas at the bindings `annotations.wgsl` declares.
fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
  let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty,
    count: None,
  };
  let storage = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Storage { read_only: true },
    has_dynamic_offset: false,
    min_binding_size: None,
  };
  let texture = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::D2,
    multisampled: false,
  };
  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Screenwide annotate overlay bindings"),
    entries: &[
      entry(
        0,
        wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
      ),
      entry(1, texture),
      entry(2, texture),
      entry(7, storage),
      entry(8, storage),
      entry(9, texture),
      entry(10, storage),
      entry(11, storage),
    ],
  })
}
