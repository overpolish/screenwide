// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn submit(
    &self,
    target: &wgpu::TextureView,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    composition: super::super::ComposedFrame,
    camera: Option<(&SourceTexture, BakeGeometry, bool, bool)>,
    picture: Option<std::sync::Arc<super::super::background_image::BackgroundImage>>,
    mut values: Constants,
    // The prepared annotations, their exposure samples, and what each counter's
    // number is: the numbers are rasterised here, at the size they are drawn.
    prepared: &PreparedArrows,
  ) -> Result<(), String> {
    let gpu = self.gpu;
    let (numbers, annotations) =
      super::super::counter_artwork::numbered_arrows(&self.counter_atlas, gpu, prepared)?;
    values.annotation_options[2] = numbers.as_ref().map_or(0, |atlas| atlas.size.0);
    values.annotation_options[3] = numbers.as_ref().map_or(0, |atlas| atlas.size.1);
    values.motion[3] = numbers.as_ref().map_or(0.0, |atlas| atlas.scale);
    let keyboard = composition
      .keyboard
      .map(|overlay| self.keyboard_cache.resolve(gpu, &overlay, settings.height))
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
    let source_view = self
      .redactor
      .apply(
        gpu,
        &mut encoder,
        source,
        &prepared.redactions,
        source.picture.is_some(),
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
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide preview bindings"),
      layout: &self.layout,
      entries: &[
        buffer(0, &self.constants),
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
      ],
    });
    {
      // A layer keeps the layers drawn under it; a whole canvas is written over
      // every pixel, so what the target held is never read.
      let load = if composition.foreground_only {
        wgpu::LoadOp::Load
      } else {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
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
      pass.set_pipeline(if composition.foreground_only {
        &self.layer_pipeline
      } else {
        &self.pipeline
      });
      pass.set_bind_group(0, &bindings, &[]);
      pass.set_viewport(
        0.0,
        0.0,
        settings.width as f32,
        settings.height as f32,
        0.0,
        1.0,
      );
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    Ok(())
  }
}
