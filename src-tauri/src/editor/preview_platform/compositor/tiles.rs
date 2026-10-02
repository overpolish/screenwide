// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotation tiles `annotation_tiles.wgsl` bins, so each canvas pixel
//! walks only the annotations that may reach it. Every annotation is tested
//! against every pixel otherwise, and on a large canvas that, not drawing
//! them, is what an annotated frame spends its time on.

use super::Gpu;

/// A tile's side in canvas pixels: small enough that a stroke's box leaves
/// most of the canvas out, large enough that binning is a few thousand
/// threads.
const TILE_SIDE: u32 = 32;
/// The binning pass's workgroup, in tiles a side.
const WORKGROUP: u32 = 8;

pub(super) struct AnnotationTiles {
  inputs: wgpu::BindGroupLayout,
  output: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  /// Every tile's set, rewritten by each draw that shows an annotation.
  masks: std::sync::Mutex<wgpu::Buffer>,
}

/// The tiles one draw bins into, as `Canvas::annotation_tiles` carries them:
/// side, grid width and height, and words a tile's set takes.
pub(super) fn grid(size: (u32, u32), total: u32) -> [u32; 4] {
  if total == 0 {
    return [TILE_SIDE, 1, 1, 1];
  }
  [
    TILE_SIDE,
    size.0.div_ceil(TILE_SIDE).max(1),
    size.1.div_ceil(TILE_SIDE).max(1),
    total.div_ceil(32),
  ]
}

impl AnnotationTiles {
  pub(super) fn new(gpu: &Gpu, module: &wgpu::ShaderModule) -> Self {
    let device = &gpu.device;
    let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
      binding,
      visibility: wgpu::ShaderStages::COMPUTE,
      ty,
      count: None,
    };
    let storage = |read_only| wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Storage { read_only },
      has_dynamic_offset: false,
      min_binding_size: None,
    };
    let inputs = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide annotation tile inputs"),
      entries: &[
        entry(
          0,
          wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
        ),
        entry(7, storage(true)),
        entry(8, storage(true)),
        entry(10, storage(true)),
      ],
    });
    let output = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("Screenwide annotation tile output"),
      entries: &[entry(0, storage(false))],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide annotation tile layout"),
      bind_group_layouts: &[Some(&inputs), Some(&output)],
      immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: Some("Screenwide annotation tiles"),
      layout: Some(&layout),
      module,
      entry_point: Some("bin_annotations"),
      compilation_options: Default::default(),
      cache: None,
    });
    Self {
      masks: std::sync::Mutex::new(masks_buffer(gpu, 4)),
      inputs,
      output,
      pipeline,
    }
  }

  /// Records binning the frame's `total` annotations into `grid`'s tiles,
  /// from the constants and lists the canvas pass is bound to, and answers
  /// the buffer that pass reads them from. A frame showing none bins
  /// nothing: no pixel asks.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn bin(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    grid: [u32; 4],
    total: u32,
    constants: &wgpu::Buffer,
    arrows: &wgpu::Buffer,
    samples: &wgpu::Buffer,
    points: &wgpu::Buffer,
  ) -> Result<wgpu::Buffer, String> {
    let bytes = u64::from(grid[1]) * u64::from(grid[2]) * u64::from(grid[3]) * 4;
    let mut masks = self
      .masks
      .lock()
      .map_err(|_| "The annotation tiles are poisoned".to_owned())?;
    if bytes > masks.size() {
      *masks = masks_buffer(gpu, bytes.next_power_of_two());
    }
    if total == 0 {
      return Ok(masks.clone());
    }
    fn bound(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
      wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
      }
    }
    let inputs = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide annotation tile inputs"),
      layout: &self.inputs,
      entries: &[
        bound(0, constants),
        bound(7, arrows),
        bound(8, samples),
        bound(10, points),
      ],
    });
    let output = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide annotation tile output"),
      layout: &self.output,
      entries: &[bound(0, &masks)],
    });
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("Screenwide annotation tiles"),
      timestamp_writes: None,
    });
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &inputs, &[]);
    pass.set_bind_group(1, &output, &[]);
    pass.dispatch_workgroups(grid[1].div_ceil(WORKGROUP), grid[2].div_ceil(WORKGROUP), 1);
    drop(pass);
    Ok(masks.clone())
  }
}

fn masks_buffer(gpu: &Gpu, size: u64) -> wgpu::Buffer {
  gpu.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Screenwide annotation tiles"),
    size,
    usage: wgpu::BufferUsages::STORAGE,
    mapped_at_creation: false,
  })
}
