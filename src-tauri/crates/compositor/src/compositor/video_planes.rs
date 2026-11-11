// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A video frame's luma and chroma planes made into the BGRA the canvas
//! samples, and a drawn canvas split back into planes for the encoder.

use screenwide_gpu::{RenderStages, Shader};

use super::pipeline::sampler;
use super::{Gpu, FORMAT};
use crate::precompiled_shaders::VIDEO_PLANES;

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/video_planes.wgsl"));

pub struct VideoPlanes {
  shader: Shader,
  from_planes: wgpu::RenderPipeline,
  luma: wgpu::RenderPipeline,
  chroma: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
}

impl VideoPlanes {
  pub fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let shader = Shader::new(
      device,
      "Screenwide video planes",
      &crate::shader_layouts::VIDEO_PLANES,
      SHADER,
      VIDEO_PLANES,
    );
    let pipeline = |fragment: &str, format: wgpu::TextureFormat| {
      shader.render_pipeline(&RenderStages {
        label: fragment,
        vertex: "vs_main",
        fragment,
        targets: &[Some(format.into())],
        primitive: Default::default(),
      })
    };
    Self {
      from_planes: pipeline("fs_from_planes", FORMAT),
      luma: pipeline("fs_luma", wgpu::TextureFormat::R8Unorm),
      chroma: pipeline("fs_chroma", wgpu::TextureFormat::Rg8Unorm),
      sampler: sampler(device, wgpu::FilterMode::Linear),
      shader,
    }
  }

  /// Draws the frame whose planes are `luma` and `chroma` into `target`, a
  /// texture the size of `luma` in the canvas's format.
  pub fn decode(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    luma: &wgpu::TextureView,
    chroma: &wgpu::TextureView,
    target: &wgpu::TextureView,
  ) {
    let bindings = self.bindings(gpu, luma, chroma);
    draw(encoder, &self.from_planes, &bindings, target);
  }

  /// Splits `canvas`, as drawn, into the `luma` and half-size `chroma`
  /// planes of a frame the encoder takes.
  pub fn encode(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    canvas: &wgpu::TextureView,
    luma: &wgpu::TextureView,
    chroma: &wgpu::TextureView,
  ) {
    let bindings = self.bindings(gpu, canvas, canvas);
    draw(encoder, &self.luma, &bindings, luma);
    draw(encoder, &self.chroma, &bindings, chroma);
  }

  fn bindings(
    &self,
    gpu: &Gpu,
    first: &wgpu::TextureView,
    second: &wgpu::TextureView,
  ) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide video planes bindings"),
      layout: self.shader.bind_group_layout(0),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(first),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(second),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    })
  }
}

fn draw(
  encoder: &mut wgpu::CommandEncoder,
  pipeline: &wgpu::RenderPipeline,
  bindings: &wgpu::BindGroup,
  target: &wgpu::TextureView,
) {
  let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide video planes pass"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view: target,
      depth_slice: None,
      resolve_target: None,
      // Every pixel is written, so what the target held is never read.
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  pass.set_pipeline(pipeline);
  pass.set_bind_group(0, bindings, &[]);
  pass.draw(0..3, 0..1);
}
