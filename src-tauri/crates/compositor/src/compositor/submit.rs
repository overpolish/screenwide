// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::canvas_pass::Variant;
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
    // The background's picture, then the one giving way while a scene's
    // background blooms in over it.
    pictures: [Option<std::sync::Arc<super::super::background_image::BackgroundImage>>; 2],
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
    // A band of an unplaced canvas lays the blur layers over its own part.
    let part = layer.band.filter(|_| layer.placement.is_none());
    let blur = super::mark_blur::Plan::of(&values, &annotations, size, part);
    if let Some(plan) = blur {
      values.annotation_blur = plan.reading();
      values.annotation_blur_origin = plan.origin();
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
    let frame = layer.frame(size);
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
        layer.source_window,
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
    let cursor_view = self.cursor_view();
    let lenses = atlas.as_ref().unwrap_or(fallback);
    let pictures = pictures
      .map(|picture| picture.map_or_else(|| fallback.clone(), |picture| picture.view.clone()));
    let bind = |constants: &wgpu::Buffer, [marks, cursor, lenses]: [&wgpu::TextureView; 3]| {
      gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenwide preview bindings"),
        layout: &self.layout,
        entries: &[
          buffer(0, constants),
          buffer(1, &self.keyboard_constants),
          texture(2, &source_view),
          texture(3, &cursor_view),
          texture(4, camera.map_or(fallback, |(camera, _, _, _)| &camera.view)),
          texture(
            5,
            keyboard
              .as_ref()
              .map_or(fallback, |(artwork, _)| &artwork.view),
          ),
          texture(6, &pictures[0]),
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
          texture(20, &pictures[1]),
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
    // A placed layer is a preview's: a pen taken up there prepares its
    // stroke's variant from what it was drawn with (`variant_choice.rs`).
    if layer.placement.is_some() {
      self
        .canvas
        .remember(layer.redaction_slot, canvas_target, kinds, features);
    }
    let variant = Variant {
      kinds,
      every,
      features,
    };
    match &focus {
      // A focused frame: what lies under the focus into the chain, blurred
      // into the target, then what lies over it laid sharp on top.
      Some(plan) => {
        let [under, over] = &self.focus.constants;
        // A band starts part way into the canvas, so its chain is held to
        // the coarse texels as a placed canvas's is.
        let moved = layer.placement.is_some() || layer.band.is_some();
        let chain_size = super::focus::FocusPlan::chain_size(frame, moved);
        let chain = self.focus.chain(gpu, chain_size, layer.redaction_slot)?;
        let clear = wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT);
        let under = bind(under, layers);
        self.canvas_pass(
          &mut encoder,
          (&chain, chain_size, clear),
          canvas_target,
          variant,
          &under,
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
          let over = bind(over, layers);
          let to = (target, frame, wgpu::LoadOp::Load);
          self.canvas_pass(&mut encoder, to, Target::Layer, variant, &over);
        }
      }
      None => {
        let bindings = bind(&self.constants, layers);
        self.canvas_pass(
          &mut encoder,
          (target, frame, load),
          canvas_target,
          variant,
          &bindings,
        );
      }
    }
    gpu.queue.submit([encoder.finish()]);
    Ok(source_view)
  }
}
