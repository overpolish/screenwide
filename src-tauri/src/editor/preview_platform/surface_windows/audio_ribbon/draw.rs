// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl AudioRibbon {
  pub(super) fn draw(&mut self) -> Result<(), String> {
    let Some(levels) = self.levels.as_ref() else {
      return Ok(());
    };
    let [red, green, blue] = crate::system_accent::accent_rgb();
    let constants = Constants {
      color: [red, green, blue, 1.0],
      flat: [self.neutral, self.neutral, self.neutral, 0.25],
      geometry: [
        self.viewport.0 as f32,
        self.viewport.1 as f32,
        5.0 * self.scale,
        2.0 * self.scale,
      ],
      style: [
        self.viewport.1 as f32 * 0.30,
        self.scale,
        self.playhead,
        self.points as f32,
      ],
    };
    // A paused waveform needs no repeated presents; recoloring or replacing
    // envelopes still invalidates it even if its playhead has not moved.
    if !self.dirty && self.last_constants == Some(constants) {
      return Ok(());
    }
    let gpu = self.shared;
    let Frame::Ready(frame) = self.surface.acquire(gpu)? else {
      return Ok(());
    };
    gpu
      .queue
      .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    let target = frame.texture.create_view(&Default::default());
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide audio ribbon"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide audio ribbon"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, levels, &[]);
      pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.queue.present(frame);
    self.dirty = false;
    self.last_constants = Some(constants);
    Ok(())
  }
}
