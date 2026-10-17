// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focus blur: everything under the topmost focus softened outside its
//! box, and everything over it drawn sharp on top.
//!
//! A focused frame is drawn in three passes. The canvas pass draws what lies
//! under the topmost focus - the background and, in the order the panes are
//! drawn, everything before the focus - into the first level of a chain the
//! size of the target. The levels below it are made from it, and the gather
//! lays the frame into the target, each pixel read from the level whose
//! texels average about as wide as its blur, so however wide the blur grows
//! it costs the same disc of samples. Last the canvas pass draws what lies
//! over the focus - everything after it, the captions, the shortcut strip -
//! on nothing, laid over the blur. What lies under the focus is what a
//! spotlight's shade in its place would darken: a camera in front of the
//! screen stays sharp under a focus on the screen, and one behind it blurs.
//!
//! How wide the blur is follows the point of the upright screen a pixel
//! shows: none inside a box, growing with the distance past its edge and, on
//! a screen turned in depth, with how far past the box the screen sits. The
//! background lies beyond the screen, so it takes the whole blur.

mod pipeline;
mod plan;

pub(super) use plan::FocusPlan;

use super::*;
use crate::annotation_gpu::GpuBuffer;
use pipeline::{draw, level, Chain};
use plan::FocusConstants;

/// Where the gather lays the blurred frame: the target, how what it held is
/// treated, whether the frame is laid over it as a layer, and the viewport
/// the canvas pass draws with.
pub(super) struct FocusTarget<'a> {
  pub(super) view: &'a wgpu::TextureView,
  pub(super) load: wgpu::LoadOp<wgpu::Color>,
  pub(super) over: bool,
  pub(super) size: (u32, u32),
}

pub(super) struct FocusBlur {
  /// The canvas constants the passes drawing what lies under the focus and
  /// over it read.
  pub(super) constants: [wgpu::Buffer; 2],
  layouts: [wgpu::BindGroupLayout; 2],
  downsample: wgpu::RenderPipeline,
  /// The gather onto a whole canvas, and over a layer.
  gather: [wgpu::RenderPipeline; 2],
  sampler: wgpu::Sampler,
  uniform: wgpu::Buffer,
  regions: GpuBuffer,
  chains: std::sync::Mutex<std::collections::HashMap<usize, Chain>>,
}

impl FocusBlur {
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
    let layouts = pipeline::layouts(device);
    let (downsample, gather) = pipeline::pipelines(device, &layouts);
    Self {
      constants: [
        uniform("Screenwide focus under canvas", size_of::<Constants>()),
        uniform("Screenwide focus over canvas", size_of::<Constants>()),
      ],
      layouts,
      downsample,
      gather,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Screenwide focus sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      uniform: uniform("Screenwide focus constants", size_of::<FocusConstants>()),
      regions: GpuBuffer::new(gpu, "Screenwide focus regions"),
      chains: std::sync::Mutex::new(std::collections::HashMap::new()),
    }
  }

  /// The first level of `slot`'s chain, made `size` first, which the canvas
  /// pass draws what lies under the focus into.
  pub(super) fn chain(
    &self,
    gpu: &Gpu,
    size: (u32, u32),
    slot: usize,
  ) -> Result<wgpu::TextureView, String> {
    let mut chains = self
      .chains
      .lock()
      .map_err(|_| "The focus chains are poisoned".to_owned())?;
    let chain = chains
      .entry(slot)
      .or_insert_with(|| pipeline::chain(gpu, size));
    if chain.size != size {
      *chain = pipeline::chain(gpu, size);
    }
    Ok(level(&chain.texture, 0))
  }

  /// Records the levels of `slot`'s chain below its first, then the gather
  /// of what `plan` blurs into `target`.
  pub(super) fn record(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    plan: &FocusPlan,
    slot: usize,
    target: FocusTarget<'_>,
  ) -> Result<(), String> {
    gpu
      .queue
      .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&plan.constants));
    let regions = self.regions.write(gpu, &plan.regions)?;
    let chains = self
      .chains
      .lock()
      .map_err(|_| "The focus chains are poisoned".to_owned())?;
    let chain = chains
      .get(&slot)
      .ok_or_else(|| "The focus chain was not drawn".to_owned())?;
    let [level_layout, gather_layout] = &self.layouts;
    let bindings = |layout: &wgpu::BindGroupLayout, entries: &[wgpu::BindingResource<'_>]| {
      let entries: Vec<_> = entries
        .iter()
        .cloned()
        .zip(0..)
        .map(|(resource, binding)| wgpu::BindGroupEntry { binding, resource })
        .collect();
      gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide focus bindings"),
        layout,
        entries: &entries,
      })
    };
    let sampler = wgpu::BindingResource::Sampler(&self.sampler);
    for at in 1..chain.texture.mip_level_count() {
      let above = level(&chain.texture, at - 1);
      let made = bindings(
        level_layout,
        &[wgpu::BindingResource::TextureView(&above), sampler.clone()],
      );
      draw(
        encoder,
        &level(&chain.texture, at),
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        (&self.downsample, &made),
        None,
      );
    }
    let gathered = bindings(
      gather_layout,
      &[
        wgpu::BindingResource::TextureView(&chain.view),
        sampler,
        self.uniform.as_entire_binding(),
        regions.as_entire_binding(),
      ],
    );
    draw(
      encoder,
      target.view,
      target.load,
      (&self.gather[usize::from(target.over)], &gathered),
      Some(target.size),
    );
    Ok(())
  }
}
