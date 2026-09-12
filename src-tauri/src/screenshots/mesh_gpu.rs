// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{mpsc, OnceLock};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::mesh::MeshGradientPoint;
use super::mesh_generator::MeshGenerator;

const MAX_POINTS: usize = 4;

/// The shader is assembled rather than kept in one file: the ported
/// generators are pure functions of a pixel and a palette, so they are
/// declared ahead of the mesh module that calls them and each file stays
/// readable on its own.
fn shader_source() -> String {
  [
    include_str!("mesh_generator_common.wgsl"),
    include_str!("mesh_generators.wgsl"),
    include_str!("mesh_generators_layered.wgsl"),
    include_str!("mesh.wgsl"),
  ]
  .join("\n")
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshUniforms {
  dimensions: [u32; 2],
  point_count: u32,
  seed: u32,
  warp_percent: f32,
  generator: u32,
  generator_color_count: u32,
  /// Canvas seconds, the value the animation reads. It sits where the struct
  /// needed a pad word anyway, so the uniform is the same size it was.
  time: f32,
  /// What `time` is multiplied by before a ported generator reads it, from
  /// the table in `mesh_generator.rs`.
  generator_speed: f32,
  /// WGSL puts a uniform `vec4` on a 16-byte boundary, which `base_color`
  /// needs once `generator_speed` has taken the word after `time`.
  padding: [f32; 3],
  base_color: [f32; 4],
  points: [[f32; 8]; MAX_POINTS],
  colors: [[f32; 4]; MAX_POINTS],
}

struct Renderer {
  device: wgpu::Device,
  pipeline: wgpu::ComputePipeline,
  queue: wgpu::Queue,
}

static RENDERER: OnceLock<Result<Renderer, String>> = OnceLock::new();

fn renderer() -> Result<&'static Renderer, String> {
  RENDERER
    .get_or_init(|| pollster::block_on(Renderer::new()))
    .as_ref()
    .map_err(Clone::clone)
}

impl Renderer {
  async fn new() -> Result<Self, String> {
    let instance = wgpu::Instance::default();
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
      })
      .await
      .map_err(|error| {
        format!("A graphics adapter is required to render mesh backgrounds: {error}")
      })?;
    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        label: Some("Screenwide mesh renderer"),
        ..Default::default()
      })
      .await
      .map_err(|error| format!("The graphics device could not be opened: {error}"))?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide mesh shader"),
      source: wgpu::ShaderSource::Wgsl(shader_source().into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: Some("Screenwide mesh pipeline"),
      layout: None,
      module: &shader,
      entry_point: Some("main"),
      compilation_options: Default::default(),
      cache: None,
    });
    Ok(Self {
      device,
      pipeline,
      queue,
    })
  }

  fn render(&self, width: u32, height: u32, uniforms: &MeshUniforms) -> Result<Vec<u8>, String> {
    let uniform_buffer = self
      .device
      .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Screenwide mesh parameters"),
        contents: bytemuck::bytes_of(uniforms),
        usage: wgpu::BufferUsages::UNIFORM,
      });
    let texture = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide mesh output"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide mesh bindings"),
      layout: &self.pipeline.get_bind_group_layout(0),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: uniform_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&view),
        },
      ],
    });
    let unpadded_bytes_per_row = width * 4;
    let bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
      * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let output = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide mesh readback"),
      size: u64::from(bytes_per_row) * u64::from(height),
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation: false,
    });
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide mesh commands"),
      });
    {
      let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Screenwide mesh pass"),
        timestamp_writes: None,
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &bind_group, &[]);
      pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
    }
    encoder.copy_texture_to_buffer(
      texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: &output,
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(bytes_per_row),
          rows_per_image: Some(height),
        },
      },
      texture.size(),
    );
    self.queue.submit([encoder.finish()]);
    let slice = output.slice(..);
    let (sender, receiver) = mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
      let _ = sender.send(result);
    });
    self
      .device
      .poll(wgpu::PollType::wait_indefinitely())
      .map_err(|error| error.to_string())?;
    receiver
      .recv()
      .map_err(|error| error.to_string())?
      .map_err(|error| error.to_string())?;
    let mapped = slice
      .get_mapped_range()
      .map_err(|error| error.to_string())?;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in mapped.chunks_exact(bytes_per_row as usize) {
      pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
    }
    drop(mapped);
    output.unmap();
    Ok(pixels)
  }
}

fn channel(value: u8) -> f32 {
  f32::from(value) / 255.0
}

fn color(value: &[u8; 4]) -> [f32; 4] {
  [channel(value[0]), channel(value[1]), channel(value[2]), 1.0]
}

/// `seconds` is where the canvas is on its timeline. Everything this renderer
/// serves is a still, so its callers pass zero; the parameter is here so a
/// test can ask for the moving picture the native backends paint.
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
  width: u32,
  height: u32,
  generator: &MeshGenerator,
  colors: &[[u8; 4]],
  points: &[MeshGradientPoint],
  seed: u32,
  warp_percent: f64,
  seconds: f64,
) -> Result<image::RgbaImage, String> {
  let mut uniforms = MeshUniforms {
    dimensions: [width, height],
    point_count: points.len() as u32,
    seed,
    warp_percent: warp_percent as f32,
    generator: generator.id,
    generator_color_count: generator.color_count as u32,
    time: seconds as f32,
    generator_speed: generator.speed,
    padding: [0.0; 3],
    base_color: color(colors.last().unwrap_or(&[0, 0, 0, u8::MAX])),
    points: [[0.0; 8]; MAX_POINTS],
    colors: [[0.0; 4]; MAX_POINTS],
  };
  // A ported generator reads its palette straight off the front of the
  // colours, and reads neither the blobs nor the warp.
  if generator.id != 0 {
    let last = colors.last().copied().unwrap_or([0, 0, 0, u8::MAX]);
    for index in 0..MAX_POINTS {
      uniforms.colors[index] = color(colors.get(index).unwrap_or(&last));
    }
    let pixels = renderer()?.render(width, height, &uniforms)?;
    return image::RgbaImage::from_raw(width, height, pixels)
      .ok_or_else(|| "The GPU returned invalid mesh pixels".to_owned());
  }
  for (index, point) in points.iter().enumerate() {
    let angle = point.rotation.to_radians() as f32;
    uniforms.points[index] = [
      point.x as f32 / 100.0,
      point.y as f32 / 100.0,
      point.radius_x as f32 / 100.0,
      point.radius_y as f32 / 100.0,
      angle.cos(),
      angle.sin(),
      0.0,
      0.0,
    ];
    uniforms.colors[index] = color(&colors[index]);
  }
  let pixels = renderer()?.render(width, height, &uniforms)?;
  image::RgbaImage::from_raw(width, height, pixels)
    .ok_or_else(|| "The GPU returned invalid mesh pixels".to_owned())
}

#[cfg(test)]
mod tests;
