// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::canvas_pipelines::Target;
use super::lens_ground::{GroundPlan, LensGround};
use super::pipeline::{buffer, texture};
use super::*;

impl Compositor {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn submit(
    &self,
    target: &wgpu::TextureView,
    source: &SourceTexture,
    size: (u32, u32),
    composition: ComposedFrame,
    camera: Option<(&SourceTexture, BakeGeometry, bool, bool)>,
    picture: Option<std::sync::Arc<super::super::background_image::BackgroundImage>>,
    mut values: Constants,
    // The prepared annotations, their exposure samples, and what each counter's
    // number is: the numbers are rasterised here, at the size they are drawn.
    prepared: &PreparedArrows,
    layer: LayerDraw,
  ) -> Result<wgpu::TextureView, String> {
    let gpu = &*self.gpu;
    let (numbers, mut annotations) = numbered_arrows(&self.counter_atlas, gpu, prepared)?;
    let images = place_images(&self.image_atlas, gpu, prepared, &mut annotations)?;
    values.annotation_options[2] = numbers.as_ref().map_or(0, |atlas| atlas.size.0);
    values.annotation_options[3] = numbers.as_ref().map_or(0, |atlas| atlas.size.1);
    values.motion[3] = numbers.as_ref().map_or(0.0, |atlas| atlas.scale);
    let keyboard = composition
      .keyboard
      .map(|overlay| self.keyboard_cache.resolve(gpu, &overlay, size.1))
      .transpose()?
      .flatten();
    let keyboard_values = keyboard
      .as_ref()
      .map_or_else(KeyboardConstants::default, |(_, values)| *values);
    gpu.queue.write_buffer(
      &self.keyboard_constants,
      0,
      bytemuck::bytes_of(&keyboard_values),
    );
    let total = values.annotation_options[1];
    values.annotation_tiles = super::tiles::grid(size, total);
    let kinds = if total == 0 {
      0
    } else {
      super::canvas_variants::kinds_of(&annotations)
    };
    let every = self
      .every_annotation_kind
      .load(std::sync::atomic::Ordering::Relaxed);
    let blur = super::mark_blur::Plan::of(&values, &annotations, size);
    if let Some(plan) = blur {
      values.annotation_blur = plan.reading();
      for (layer, constants) in self.mark_blur.constants.iter().enumerate() {
        gpu.queue.write_buffer(
          constants,
          0,
          bytemuck::bytes_of(&plan.drawing(values, layer)),
        );
      }
    }
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&values));
    let focus = super::focus::FocusPlan::of(&values, prepared);
    let features = super::canvas_features::of(&values, focus.is_some(), keyboard.is_some());
    if let Some(plan) = &focus {
      for (part, constants) in self.focus.constants.iter().enumerate() {
        gpu
          .queue
          .write_buffer(constants, 0, bytemuck::bytes_of(&plan.pass(values, part)));
      }
    }
    let frame = layer.placement.map_or(size, |placement| placement.target);
    // Where each lens's ground lies, written into its record before upload.
    let focused = focus.is_some();
    let grounds = GroundPlan::of(kinds, &mut annotations, prepared, &values, focused, frame);
    let annotation_buffer = self.annotations.write(gpu, &annotations)?;
    let sample_buffer = self.samples.write(gpu, &prepared.samples)?;
    let points_buffer = self.annotation_points.write(gpu, &prepared.points)?;
    let text_buffer = self.annotation_text.write(gpu, &prepared.text)?;
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide preview frame"),
      });
    let tile_buffer = self.tiles.bin(
      gpu,
      &mut encoder,
      values.annotation_tiles,
      total,
      &self.constants,
      &annotation_buffer,
      &sample_buffer,
      &points_buffer,
    )?;
    let source_view = self
      .redactor
      .apply(
        gpu,
        &mut encoder,
        source,
        &prepared.redactions,
        layer.redaction_slot,
        layer.retained_source,
      )?
      .unwrap_or_else(|| source.view.clone());
    // A lens on a layer laid over others shows them through a copy of the
    // target as it stands, taken before this layer draws over it.
    let under = match &grounds {
      Some(plan) if composition.foreground_only => {
        self
          .lens_under
          .record(gpu, &mut encoder, target.texture(), plan.under)?
      }
      _ => None,
    };
    let atlas = grounds
      .as_ref()
      .map(|plan| self.lens_ground.atlas(gpu, plan))
      .transpose()?;
    let fallback = &self.fallback_view;
    let lenses = atlas.as_ref().unwrap_or(fallback);
    let bind = |constants: &wgpu::Buffer, [marks, cursor, lenses]: [&wgpu::TextureView; 3]| {
      gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide preview bindings"),
        layout: &self.layout,
        entries: &[
          buffer(0, constants),
          buffer(1, &self.keyboard_constants),
          texture(2, &source_view),
          texture(3, &self.cursor_view),
          texture(4, camera.map_or(fallback, |(camera, _, _, _)| &camera.view)),
          texture(
            5,
            keyboard
              .as_ref()
              .map_or(fallback, |(artwork, _)| &artwork.view),
          ),
          texture(
            6,
            picture.as_ref().map_or(fallback, |picture| &picture.view),
          ),
          buffer(7, &annotation_buffer),
          buffer(8, &sample_buffer),
          texture(9, numbers.as_ref().map_or(fallback, |atlas| &atlas.view)),
          buffer(10, &points_buffer),
          buffer(11, &text_buffer),
          wgpu::BindGroupEntry {
            binding: 12,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
          wgpu::BindGroupEntry {
            binding: 13,
            resource: wgpu::BindingResource::Sampler(&self.point_sampler),
          },
          buffer(14, &tile_buffer),
          texture(15, marks),
          texture(16, cursor),
          texture(17, images.as_ref().unwrap_or(fallback)),
          texture(18, under.as_ref().unwrap_or(fallback)),
          texture(19, lenses),
        ],
      })
    };
    // The grounds are drawn by the same variant as the layers, ahead of each
    // pass that reads them: the marks' blur layer, which they are drawn
    // under without, then the canvas, with it.
    let delta = || self.canvas.get(Target::Delta, kinds, every, features);
    let draw_grounds =
      |encoder: &mut wgpu::CommandEncoder, slot: usize, [marks, cursor]: [_; 2]| {
        if let (Some(plan), Some(atlas)) = (&grounds, &atlas) {
          self.lens_ground.write(gpu, slot, values);
          let bindings = bind(&self.lens_ground.constants[slot], [marks, cursor, fallback]);
          LensGround::draw(encoder, atlas, plan, &delta(), &bindings);
        }
      };
    let blurred = blur
      .map(|plan| {
        draw_grounds(&mut encoder, 0, [fallback, fallback]);
        let [marks, cursor] = &self.mark_blur.constants;
        self.mark_blur.record(
          gpu,
          &mut encoder,
          &delta(),
          [
            &bind(marks, [fallback, fallback, lenses]),
            &bind(cursor, [fallback, fallback, lenses]),
          ],
          plan,
        )
      })
      .transpose()?;
    let layers = blurred
      .as_ref()
      .map_or([fallback, fallback], |[marks, cursor]| [marks, cursor]);
    draw_grounds(&mut encoder, 1, layers);
    let [marks, cursor] = layers;
    let layers = [marks, cursor, lenses];
    // A layer keeps the layers drawn under it; a whole canvas is written over
    // every pixel, so what the target held is never read. A placed canvas
    // leaves the target as it was past its edge, under whatever lies there,
    // so it is laid over the target as a layer is.
    let blended = composition.foreground_only || layer.placement.is_some();
    let load = if layer.clear {
      wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    } else {
      wgpu::LoadOp::Load
    };
    // A placed canvas keeps the target's own pixels, which the shader maps
    // back to canvas pixels, and covers the whole target, so what lies past
    // its edge is drawn too. A viewport would scale it instead, but one is
    // capped at the largest texture size, and a canvas zoomed far in is
    // drawn larger than that.
    let canvas_target = if blended {
      Target::Layer
    } else {
      Target::Canvas
    };
    let canvas_pass = |encoder: &mut wgpu::CommandEncoder,
                       view: &wgpu::TextureView,
                       viewport: (u32, u32),
                       load: wgpu::LoadOp<wgpu::Color>,
                       pipeline: Target,
                       bindings: &wgpu::BindGroup| {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide preview canvas"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_viewport(0.0, 0.0, viewport.0 as f32, viewport.1 as f32, 0.0, 1.0);
      pass.set_pipeline(&self.canvas.get(pipeline, kinds, every, features));
      pass.set_bind_group(0, bindings, &[]);
      pass.draw(0..3, 0..1);
    };
    match &focus {
      // A focused frame: what lies under the focus into the chain, blurred
      // into the target, then what lies over it laid sharp on top.
      Some(plan) => {
        let [under, over] = &self.focus.constants;
        let chain_size = super::focus::FocusPlan::chain_size(frame, layer.placement.is_some());
        let chain = self.focus.chain(gpu, chain_size, layer.redaction_slot)?;
        canvas_pass(
          &mut encoder,
          &chain,
          chain_size,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          canvas_target,
          &bind(under, layers),
        );
        self.focus.record(
          gpu,
          &mut encoder,
          plan,
          layer.redaction_slot,
          super::focus::FocusTarget {
            view: target,
            load,
            over: blended,
            size: frame,
          },
        )?;
        if plan.draws_over(&values, &annotations, keyboard.is_some()) {
          canvas_pass(
            &mut encoder,
            target,
            frame,
            wgpu::LoadOp::Load,
            Target::Layer,
            &bind(over, layers),
          );
        }
      }
      None => canvas_pass(
        &mut encoder,
        target,
        frame,
        load,
        canvas_target,
        &bind(&self.constants, layers),
      ),
    }
    gpu.queue.submit([encoder.finish()]);
    Ok(source_view)
  }
}
