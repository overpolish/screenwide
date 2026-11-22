// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas's generator drawn once a frame into a picture at half the
//! resolution the canvas is shown at, which the canvas pass reads instead of
//! evaluating the generator at every pixel (`background_field.wgsl`). A
//! generator is the costliest thing most frames draw, and the smooth ones look
//! the same at half resolution scaled back up; which those are is the
//! generator table's `half_resolution`. Only a compositor whose owner draws
//! moving pictures does this (`Compositor::draw_generators_at_half_resolution`).
//! The picture is never larger than the canvas: a canvas the workspace shows
//! zoomed in further than that evaluates the generator at every pixel, as
//! does the background giving way while a scene's blooms in.

use super::pipeline::buffer;
use super::{Compositor, Constants};
use crate::precompiled_shaders::BACKGROUND_FIELD;
use crate::shader_layouts;
use screenwide_gpu::{Gpu, RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/background_field.wgsl"));

/// Held at sixteen bits a channel, so the canvas reads it back without the
/// steps eight would leave in a slow gradient.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The twin of `Field` in `background_field.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct FieldConstants {
  size: [f32; 4],
  colors: [[f32; 4]; 4],
  shift: [f32; 4],
  timing: [f32; 4],
  style: [u32; 4],
}

impl FieldConstants {
  /// What the field draws for the canvas `values` describe, where the canvas
  /// may read its generator from one, marked in `values` so the canvas pass
  /// reads it: a generator that keeps at half resolution, no picture over
  /// it, a background drawn at all, and a canvas shown no larger than twice
  /// its own size, one field texel covering two drawn pixels each way.
  fn plan(values: &mut Constants) -> Option<Self> {
    let generator = values.background_options[1];
    let keeps = screenwide_output::mesh_generator::every_generator()
      .iter()
      .any(|candidate| candidate.id == generator && candidate.half_resolution);
    let [width, height, ..] = values.output_source;
    // Canvas pixels to a field texel.
    let texel = 2.0 * values.motion[2];
    let drawn = values.options[1] != 0
      && values.background_options[0] == 0
      && values.cursor_options[3] == 0
      && keeps
      && texel >= 1.0
      && width >= 1.0
      && height >= 1.0;
    values.background_options[3] |= u32::from(drawn);
    drawn.then(|| Self {
      size: [
        width,
        height,
        (width / texel).ceil(),
        (height / texel).ceil(),
      ],
      colors: values.mesh_colors,
      shift: values.mesh_points[0],
      timing: [values.motion[0], values.motion[1], 0.0, 0.0],
      style: [
        generator,
        values.background_options[2],
        values.options[0],
        0,
      ],
    })
  }
}

pub(super) struct BackgroundField {
  pipeline: wgpu::RenderPipeline,
  bindings: wgpu::BindGroup,
  constants: wgpu::Buffer,
  /// The picture and what it was last drawn with: a canvas drawn again with
  /// the same, as a paused preview is, reads it as it stands.
  held: std::sync::Mutex<Option<(wgpu::Texture, FieldConstants)>>,
  /// Whether this compositor's owner draws its canvases this way
  /// (`Compositor::draw_generators_at_half_resolution`).
  allowed: std::sync::atomic::AtomicBool,
}

impl Compositor {
  /// Has this compositor draw a smooth generator at half the resolution its
  /// canvas is shown at, as an export and a preview of a recording want, who
  /// draw it again for every frame. A still is drawn once, where the saving
  /// is nothing and it should be just the picture the background renderer
  /// paints, so a compositor evaluates the generator at every pixel unless
  /// its owner asks for this.
  pub fn draw_generators_at_half_resolution(&self) {
    self
      .background_field
      .allowed
      .store(true, std::sync::atomic::Ordering::Relaxed);
  }
}

impl BackgroundField {
  pub(super) fn new(device: &wgpu::Device) -> Self {
    let shader = Shader::new(
      device,
      "Screenwide background field",
      &shader_layouts::BACKGROUND_FIELD,
      SHADER,
      BACKGROUND_FIELD,
    );
    let pipeline = shader.render_pipeline(&RenderStages {
      label: "Screenwide background field pipeline",
      vertex: "vs_main",
      fragment: "fs_main",
      targets: &[Some(wgpu::ColorTargetState {
        format: FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
      primitive: Default::default(),
    });
    let constants = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Screenwide background field"),
      size: size_of::<FieldConstants>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide background field bindings"),
      layout: shader.bind_group_layout(0),
      entries: &[buffer(0, &constants)],
    });
    Self {
      pipeline,
      bindings,
      constants,
      held: std::sync::Mutex::new(None),
      allowed: std::sync::atomic::AtomicBool::new(false),
    }
  }

  /// What the field draws for the canvas `values` describe, marked in
  /// `values`, where this compositor draws one at all; see `FieldConstants`.
  pub(super) fn plan(&self, values: &mut Constants) -> Option<FieldConstants> {
    if !self.allowed.load(std::sync::atomic::Ordering::Relaxed) {
      return None;
    }
    FieldConstants::plan(values)
  }

  /// Records the field `field` describes into `encoder`, unless the picture
  /// held already shows it, and answers a view of it.
  pub(super) fn record(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    field: FieldConstants,
  ) -> Result<wgpu::TextureView, String> {
    let mut held = self
      .held
      .lock()
      .map_err(|_| "The background field is poisoned".to_owned())?;
    let size = (field.size[2] as u32, field.size[3] as u32);
    if let Some((texture, drawn)) = held.as_ref() {
      if *drawn == field {
        return Ok(texture.create_view(&Default::default()));
      }
    }
    let fits = held
      .as_ref()
      .is_some_and(|(texture, _)| (texture.width(), texture.height()) == size);
    let texture = match held.take() {
      Some((texture, _)) if fits => texture,
      _ => gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenwide background field"),
        size: wgpu::Extent3d {
          width: size.0,
          height: size.1,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      }),
    };
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&field));
    let view = texture.create_view(&Default::default());
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide background field"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &self.bindings, &[]);
      pass.draw(0..3, 0..1);
    }
    *held = Some((texture, field));
    Ok(view)
  }
}
