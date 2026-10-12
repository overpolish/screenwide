// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects on the GPU (`camera_effects.wgsl`): a camera frame
//! and its matte in, the camera as its effects draw it out, a picture the
//! size of the frame that the canvas samples where it would have sampled the
//! frame. The preview runs it as each frame is drawn, so a change to an
//! effect is a redraw away; the export runs it on every frame it writes.

mod resources;

use std::collections::HashMap;
use std::sync::Mutex;

use wgpu::util::DeviceExt;

use self::resources::{bind_group_layout, cover, upload, Slot, Uniform};
use super::live::LiveCameraEffects;
use super::CameraBackground;
use screenwide_annotations::pin::LumaFrame;
use screenwide_compositor::compositor::{Compositor, SourceTexture, FORMAT};
use screenwide_gpu::Gpu;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/camera_effects.wgsl"));

/// The blurred room's deviation as a share of the frame's shorter side, and
/// the most it may be: the redaction blur reaches three deviations, at most
/// 160 pixels.
const ROOM_BLUR_SHARE: f32 = 0.025;
const ROOM_BLUR_LARGEST: f32 = 53.0;

/// How blurred the room behind the person is, in pixels of a frame `size`.
pub(crate) fn room_blur_deviation(size: (u32, u32)) -> f32 {
  (size.0.min(size.1) as f32 * ROOM_BLUR_SHARE).min(ROOM_BLUR_LARGEST)
}

/// The views one pass reads: its source, the matte, and what is behind.
type Inputs<'a> = (
  &'a wgpu::TextureView,
  &'a wgpu::TextureView,
  &'a wgpu::TextureView,
);

pub(crate) struct EffectsPass {
  gpu: &'static Gpu,
  layout: wgpu::BindGroupLayout,
  effects: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
  /// One opaque white texel, bound where a pass reads no matte or backdrop.
  blank: wgpu::TextureView,
  slots: Mutex<HashMap<usize, Slot>>,
  /// The picture on the GPU, and the effects' generation it was made for.
  picture: Mutex<Option<(u64, wgpu::Texture)>>,
}

impl EffectsPass {
  pub(crate) fn new(gpu: &'static Gpu) -> Self {
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Screenwide camera effects shader"),
      source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = bind_group_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide camera effects layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = |entry_point: &str| {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry_point),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_main"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some(entry_point),
          compilation_options: Default::default(),
          targets: &[Some(FORMAT.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    };
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Screenwide camera effects sampler"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });
    let blank = upload(
      gpu,
      wgpu::TextureFormat::Rgba8Unorm,
      (1, 1),
      &[255; 4],
      "blank",
    )
    .create_view(&Default::default());
    Self {
      gpu,
      effects: pipeline("fs_effects"),
      layout,
      sampler,
      blank,
      slots: Mutex::new(HashMap::new()),
      picture: Mutex::new(None),
    }
  }

  /// `camera` drawn with `effects`, its matte `matte`, into a picture of its
  /// own size that stays readable until `slot` is drawn again. A background
  /// taken away with no matte to hand is drawn as recorded. A blurred room
  /// is blurred by `compositor` as a blur redaction is.
  pub(crate) fn apply(
    &self,
    compositor: &Compositor,
    camera: &SourceTexture,
    matte: Option<&LumaFrame>,
    effects: &LiveCameraEffects,
    slot: usize,
  ) -> Result<SourceTexture, String> {
    let mut slots = self
      .slots
      .lock()
      .map_err(|_| "The camera effects are poisoned".to_owned())?;
    if slots.get(&slot).is_none_or(|held| held.size != camera.size) {
      slots.insert(slot, Slot::new(self.gpu, camera.size));
    }
    let index = slot;
    let slot = slots.get_mut(&slot).expect("the slot is held");
    let background = match (effects.background, matte) {
      (CameraBackground::Kept, _) | (_, None) => 0,
      (CameraBackground::Transparent, Some(_)) => 1,
      (CameraBackground::Blurred, Some(_)) => 2,
      (CameraBackground::Image, Some(_)) => 3,
    };
    let matte_view = match matte {
      Some(matte) if background != 0 => slot.matte(self.gpu, matte, effects.generation),
      _ => self.blank.clone(),
    };
    let grade = effects.grade;
    let uniform = |target: (u32, u32), picture: [f32; 4]| Uniform {
      mode: [
        background,
        u32::from(grade.is_some()),
        u32::from(effects.enhance),
        0,
      ],
      gain: grade.map_or([1.0; 4], |grade| {
        let [r, g, b] = grade.gains;
        [r, g, b, 1.0]
      }),
      picture,
      target_size: [target.0 as f32, target.1 as f32, 0.0, 0.0],
    };
    let whole = [1.0, 1.0, 0.0, 0.0];
    let mut encoder = self
      .gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide camera effects"),
      });
    let blank = &self.blank;
    let (backdrop, picture) = match background {
      2 => (
        compositor.blurred_whole(camera, room_blur_deviation(camera.size), index)?,
        whole,
      ),
      3 => match effects.picture.as_deref() {
        Some(picture) => (
          self.picture(effects.generation, picture),
          cover(picture, camera.size),
        ),
        None => (blank.clone(), whole),
      },
      _ => (blank.clone(), whole),
    };
    let main = uniform(camera.size, picture);
    let inputs = (&camera.view, &matte_view, &backdrop);
    self.pass(&mut encoder, &self.effects, inputs, &main, &slot.drawn);
    self.gpu.queue.submit([encoder.finish()]);
    let cut_out = background == 1;
    if cut_out {
      compositor.fill_mips(&slot.out.texture);
    }
    // Marked so the canvas enlarges it along its edges where it draws it
    // larger, rather than stretching it, and casts its shadow from the
    // person where nothing is behind them.
    Ok(SourceTexture {
      enhanced: effects.enhance,
      cut_out,
      ..slot.out.clone()
    })
  }

  /// The picture on the GPU, uploaded once for each set of effects.
  fn picture(&self, generation: u64, picture: &::image::RgbaImage) -> wgpu::TextureView {
    let mut held = self
      .picture
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, texture)) = held.as_ref() {
      if *at == generation {
        return texture.create_view(&Default::default());
      }
    }
    let texture = upload(
      self.gpu,
      wgpu::TextureFormat::Rgba8Unorm,
      picture.dimensions(),
      picture.as_raw(),
      "Screenwide camera background picture",
    );
    let view = texture.create_view(&Default::default());
    *held = Some((generation, texture));
    view
  }

  fn pass(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    (source, matte, backdrop): Inputs<'_>,
    uniform: &Uniform,
    target: &wgpu::TextureView,
  ) {
    let device = &self.gpu.device;
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Screenwide camera effects uniform"),
      contents: bytemuck::bytes_of(uniform),
      usage: wgpu::BufferUsages::UNIFORM,
    });
    let entry = |binding, resource| wgpu::BindGroupEntry { binding, resource };
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide camera effects bindings"),
      layout: &self.layout,
      entries: &[
        entry(0, wgpu::BindingResource::TextureView(source)),
        entry(1, wgpu::BindingResource::TextureView(matte)),
        entry(2, wgpu::BindingResource::TextureView(backdrop)),
        entry(3, wgpu::BindingResource::Sampler(&self.sampler)),
        entry(4, buffer.as_entire_binding()),
      ],
    });
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Screenwide camera effects pass"),
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
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bindings, &[]);
    pass.draw(0..3, 0..1);
  }
}
