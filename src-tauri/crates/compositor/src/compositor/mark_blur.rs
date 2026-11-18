// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spotlights' blur of the annotations under their shade and of the
//! cursor.
//!
//! The picture under a blurring spotlight arrives blurred: the source is
//! blurred before the canvas is drawn. The annotations and the cursor are
//! drawn by the canvas pass itself, so their blur is laid here, into two
//! layers over the whole canvas. One holds what the annotations under the
//! shade add to the canvas, as the difference from the canvas without them; a
//! Gaussian is linear, so blurring the difference blurs the annotations
//! without blurring the picture again. The other holds the cursor alone. Two
//! passes blur both along the rows and then along the columns, and the canvas
//! pass reads them where the blur reaches.
//!
//! The layers' resolution follows the blur rather than the zoom: two texels
//! to a deviation, which loses nothing a Gaussian keeps, so a canvas zoomed
//! far in costs no more to blur.
//!
//! The run of annotations above the camera is drawn over the run below it, so
//! where both have spotlights, the shade above the camera is the one that
//! blurs what lies under it; the shade below it only darkens.

mod pipeline;
mod plan;

pub(super) use plan::{reach, Plan};

use std::sync::Mutex;

use super::*;

/// What the layers hold: the annotations' layer is a signed difference, kept
/// beyond eight bits so a faint edge survives the blur.
pub(super) const LAYER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The twin of `MarkBlur` in `annotation_blur.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Axis {
  direction: [i32; 2],
  sigma: f32,
  reach: i32,
}

/// The two layers, then the textures their rows are blurred into, all one
/// size.
type Targets = ((u32, u32), [wgpu::TextureView; 4]);

pub(super) struct MarkBlur {
  /// The canvas constants the passes drawing the annotations' layer and the
  /// cursor's read.
  pub(super) constants: [wgpu::Buffer; 2],
  axes: [wgpu::Buffer; 2],
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  targets: Mutex<Option<Targets>>,
}

impl MarkBlur {
  pub(super) fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let uniform = |label, size: usize| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };
    let (pipeline, layout) = pipeline::pipeline(device);
    Self {
      constants: [
        uniform("Screenwide mark blur canvas", size_of::<Constants>()),
        uniform("Screenwide cursor blur canvas", size_of::<Constants>()),
      ],
      axes: [
        uniform("Screenwide mark blur rows", size_of::<Axis>()),
        uniform("Screenwide mark blur columns", size_of::<Axis>()),
      ],
      pipeline,
      layout,
      targets: Mutex::new(None),
    }
  }

  /// Records the layers `plan` lays, each drawn by `delta` with its
  /// `bindings`, whose constants are [`Self::constants`], or left empty where
  /// it has nothing to hold; then both blurred along both axes. Answers the
  /// blurred annotations' layer and the blurred cursor's.
  pub(super) fn record(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    delta: &wgpu::RenderPipeline,
    bindings: [&wgpu::BindGroup; 2],
    plan: Plan,
  ) -> Result<[wgpu::TextureView; 2], String> {
    let [marks, cursor, marks_rows, cursor_rows] = self.targets(gpu, plan.size)?;
    let drawn = [plan.spotlight.is_some(), plan.cursor];
    for ((layer, binding), drawn) in [&marks, &cursor].into_iter().zip(bindings).zip(drawn) {
      pipeline::full_pass(
        encoder,
        &[layer],
        drawn.then_some((delta, binding)),
        "Screenwide blur layer",
      );
    }
    let reach = (plan.sigma * 3.0).ceil() as i32;
    let axes = [
      ([1, 0], [&marks, &cursor], [&marks_rows, &cursor_rows]),
      ([0, 1], [&marks_rows, &cursor_rows], [&marks, &cursor]),
    ];
    for (buffer, (direction, from, into)) in self.axes.iter().zip(axes) {
      gpu.queue.write_buffer(
        buffer,
        0,
        bytemuck::bytes_of(&Axis {
          direction,
          sigma: plan.sigma,
          reach,
        }),
      );
      let texture = |binding, view| wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
      };
      let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide mark blur pass"),
        layout: &self.layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
          },
          texture(1, from[0]),
          texture(2, from[1]),
        ],
      });
      pipeline::full_pass(
        encoder,
        &into,
        Some((&self.pipeline, &bindings)),
        "Screenwide mark blur axis",
      );
    }
    Ok([marks, cursor])
  }

  fn targets(&self, gpu: &Gpu, size: (u32, u32)) -> Result<[wgpu::TextureView; 4], String> {
    let mut targets = self
      .targets
      .lock()
      .map_err(|_| "The mark blur's layers are poisoned".to_owned())?;
    if targets.as_ref().is_none_or(|(held, _)| *held != size) {
      let texture = |label| pipeline::layer_texture(gpu, size, label);
      *targets = Some((
        size,
        [
          texture("Screenwide mark blur layer"),
          texture("Screenwide cursor blur layer"),
          texture("Screenwide mark blur rows"),
          texture("Screenwide cursor blur rows"),
        ],
      ));
    }
    Ok(
      targets
        .as_ref()
        .expect("the layers were just made")
        .1
        .clone(),
    )
  }
}
