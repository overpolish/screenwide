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

use std::sync::Mutex;

use super::*;
use crate::editor::annotations::AnnotationKind;
use crate::editor::preview_platform::annotation_gpu::PreviewArrow;

/// What the layers hold: the annotations' layer is a signed difference, kept
/// beyond eight bits so a faint edge survives the blur.
pub(super) const LAYER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// How many layer texels one deviation of the blur spans.
const TEXELS_PER_DEVIATION: f32 = 2.0;

/// The longest side the layers are laid at, which only a blur narrower than
/// half a canvas pixel on a very large canvas reaches.
const LARGEST_SIDE: f32 = 4096.0;

/// The `annotation_blur` modes of the canvas pass that draws each layer.
const DRAWING_MARKS: f32 = 1.0;
const DRAWING_CURSOR: f32 = 3.0;

/// The twin of `MarkBlur` in `annotation_blur.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Axis {
  direction: [i32; 2],
  sigma: f32,
  reach: i32,
}

/// Where and how one draw's layers are laid.
#[derive(Clone, Copy)]
pub(super) struct Plan {
  /// Layer texels per canvas pixel.
  scale: f32,
  size: (u32, u32),
  /// The deviation in layer texels.
  sigma: f32,
  /// The spotlight whose shade the blurred annotations lie under, where any
  /// annotation does.
  spotlight: Option<u32>,
  cursor: bool,
}

impl Plan {
  /// The layers `values` call for over `arrows` on a canvas `size`, or
  /// `None` where no spotlight blurs or there is nothing for it to blur.
  pub(super) fn of(values: &Constants, arrows: &[PreviewArrow], size: (u32, u32)) -> Option<Self> {
    let [_, _, deviation, strength] = values.cursor_blur;
    // A deviation that is not a number blurs nothing, as the shader reads it.
    let blurs = deviation.is_finite() && deviation >= 0.5;
    if !blurs || strength <= 0.0 || size.0 == 0 || size.1 == 0 {
      return None;
    }
    let below = (values.annotation_options[0] as usize).min(arrows.len());
    let total = (values.annotation_options[1] as usize).min(arrows.len());
    // While a scene crosses the order over, the blur layer follows the
    // nearer order, as the shader draws that layer.
    let camera_between = values.camera_effects[1] != 0.0 && values.camera_effects[3] >= 0.5;
    let topmost = |first: usize, last: usize| {
      (first..last)
        .rev()
        .find(|&index| arrows[index].kind == AnnotationKind::Spotlight.raw())
    };
    let spotlight = match topmost(below, total) {
      Some(index) => (index > 0 || camera_between).then_some(index),
      None => topmost(0, below).filter(|&index| index > 0),
    };
    let cursor = values.cursor_options[1] != 0;
    if spotlight.is_none() && !cursor {
      return None;
    }
    let scale = (TEXELS_PER_DEVIATION / deviation).min(LARGEST_SIDE / size.0.max(size.1) as f32);
    let side = |length: u32| ((length as f32 * scale).ceil() as u32).max(1);
    Some(Self {
      scale,
      size: (side(size.0), side(size.1)),
      sigma: deviation * scale,
      spotlight: spotlight.map(|index| index as u32),
      cursor,
    })
  }

  fn spotlight_word(self) -> f32 {
    self.spotlight.map_or(-1.0, |index| index as f32)
  }

  /// The canvas pass's `annotation_blur` row while it reads the layers.
  pub(super) fn reading(self) -> [f32; 4] {
    [
      self.scale,
      2.0,
      self.spotlight_word(),
      f32::from(u8::from(self.cursor)),
    ]
  }

  /// `values` as the pass that draws the annotations' layer (`0`) or the
  /// cursor's (`1`) takes them: the whole canvas laid over the layer, every
  /// edge feathered over one of its texels.
  pub(super) fn drawing(self, mut values: Constants, layer: usize) -> Constants {
    let mode = if layer == 0 {
      DRAWING_MARKS
    } else {
      DRAWING_CURSOR
    };
    values.placement = [0.0, 0.0, 1.0 / self.scale, 1.0 / self.scale];
    values.motion[2] = 1.0 / self.scale;
    values.annotation_blur = [self.scale, mode, self.spotlight_word(), 0.0];
    values
  }
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
    let layout = pipeline::bind_group_layout(device);
    Self {
      constants: [
        uniform("Screenwide mark blur canvas", size_of::<Constants>()),
        uniform("Screenwide cursor blur canvas", size_of::<Constants>()),
      ],
      axes: [
        uniform("Screenwide mark blur rows", size_of::<Axis>()),
        uniform("Screenwide mark blur columns", size_of::<Axis>()),
      ],
      pipeline: pipeline::pipeline(device, &layout),
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
