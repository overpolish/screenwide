// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
    let gpu = self.gpu;
    let (numbers, mut annotations) = numbered_arrows(&self.counter_atlas, gpu, prepared)?;
    let stickers = place_stickers(&self.sticker_atlas, gpu, prepared, &mut annotations)?;
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
    let fallback = &self.fallback_view;
    fn texture(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
      wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
      }
    }
    fn buffer(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
      wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
      }
    }
    let bind = |constants: &wgpu::Buffer, [marks, cursor]: [&wgpu::TextureView; 2]| {
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
          texture(17, stickers.as_ref().unwrap_or(fallback)),
        ],
      })
    };
    let blurred = blur
      .map(|plan| {
        let [marks, cursor] = &self.mark_blur.constants;
        self.mark_blur.record(
          gpu,
          &mut encoder,
          self.canvas.delta(&gpu.device),
          [
            &bind(marks, [fallback, fallback]),
            &bind(cursor, [fallback, fallback]),
          ],
          plan,
        )
      })
      .transpose()?;
    let bindings = bind(
      &self.constants,
      blurred
        .as_ref()
        .map_or([fallback, fallback], |[marks, cursor]| [marks, cursor]),
    );
    {
      // A layer keeps the layers drawn under it; a whole canvas is written over
      // every pixel, so what the target held is never read.
      let load = if layer.clear {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
      } else {
        wgpu::LoadOp::Load
      };
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide preview canvas"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      // A placed canvas keeps the target's own pixels, which the shader maps
      // back to canvas pixels, and is cut to its rectangle. A viewport would
      // scale it instead, but one is capped at the largest texture size, and
      // a canvas zoomed far in is drawn larger than that.
      let drawn = match layer.placement {
        None => Some((size, None)),
        Some(placement) => placement
          .scissor()
          .map(|scissor| (placement.target, Some(scissor))),
      };
      if let Some(((width, height), scissor)) = drawn {
        pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        if let Some([x, y, width, height]) = scissor {
          pass.set_scissor_rect(x, y, width, height);
        }
        pass.set_pipeline(
          self
            .canvas
            .get(&gpu.device, composition.foreground_only, total == 0),
        );
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..3, 0..1);
      }
    }
    gpu.queue.submit([encoder.finish()]);
    Ok(source_view)
  }
}
