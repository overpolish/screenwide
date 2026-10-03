// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing the retained scene into the workspace drawable.

use super::loupe::NativeMagnifier;
use super::WorkspaceScene;
use crate::editor::preview_platform::compositor::{ComposedFrame, LayerDraw, LayerPlacement};
use crate::editor::preview_platform::surface_macos::NativeWorkspacePlacement;

impl WorkspaceScene {
  /// Whether the scene holds `count` layers, one for each placement a redraw
  /// would pass. A redraw that cannot draw must not take a drawable.
  pub(super) fn ready(&self, count: usize) -> bool {
    self
      .state()
      .is_ok_and(|state| count > 0 && state.layers.len() == count)
  }

  /// Draws every layer into `target` at its placement, in drawable pixels,
  /// the first at the bottom, and the crop magnifier over them all.
  pub(super) fn draw(
    &self,
    target: &wgpu::Texture,
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
  ) -> Result<(), String> {
    let state = self.state()?;
    if state.layers.len() != placements.len() {
      return Err("The workspace placements do not match its layers".to_owned());
    }
    let gpu = state.compositor.gpu();
    let size = (target.width(), target.height());
    let view = target.create_view(&Default::default());
    clear(gpu, &view);
    let mut loupe = None;
    for (index, (layer, placement)) in state.layers.iter().zip(placements).enumerate() {
      if placement.width == 0 || placement.height == 0 {
        continue;
      }
      let geometry = &layer.geometry;
      let mut prepared = layer
        .annotations
        .placed(geometry.placement, layer.hover, None);
      // The canvas is drawn into its placement, which is rarely its own size:
      // zoomed out, one drawn pixel covers several canvas pixels, and an edge
      // feathered over one canvas pixel would land inside a single drawn one.
      prepared.pixel_scale = (geometry.size.0 as f32 / placement.width as f32)
        .max(geometry.size.1 as f32 / placement.height as f32);
      let camera = layer
        .camera
        .as_ref()
        .filter(|camera| camera.geometry.frame_width > 0)
        .and_then(|camera| {
          let picture = camera.picture.as_ref()?;
          let mut geometry = camera.geometry;
          (geometry.output_width, geometry.output_height) = layer.geometry.size;
          Some((camera, picture, geometry))
        })
        .map(|(camera, picture, geometry)| {
          Ok::<_, String>((
            camera.drawn(&state.compositor, picture, index)?,
            geometry,
            camera.drop_shadow,
            camera.on_top,
          ))
        })
        .transpose()?;
      // The bottom layer is drawn whole even when it is only a foreground:
      // there is nothing under it to lay it over.
      let picture = state.compositor.draw_layer(
        &view,
        &layer.source.texture,
        &layer.settings,
        geometry,
        ComposedFrame {
          cursor: layer.cursor,
          keyboard: layer.keyboard.filter(|keyboard| keyboard.key_count > 0),
          foreground_only: layer.foreground_only && index > 0,
          seconds: layer.seconds,
        },
        camera
          .as_ref()
          .map(|(picture, geometry, shadow, on_top)| (picture, *geometry, *shadow, *on_top)),
        None,
        &prepared,
        LayerDraw {
          placement: Some(LayerPlacement {
            rect: [
              placement.x as f32,
              placement.y as f32,
              placement.width as f32,
              placement.height as f32,
            ],
            target: size,
          }),
          clear: false,
          redaction_slot: index,
          // The scene holds every picture it draws, so nothing writes into one
          // while it is drawn from.
          retained_source: true,
        },
      )?;
      let magnified = magnifier.filter(|magnifier| {
        loupe.is_none()
          && magnifier.active != 0
          && if magnifier.sample_camera != 0 {
            layer.pane_index == magnifier.pane_index
          } else {
            layer.layer_id == magnifier.layer_id
          }
      });
      if let Some(magnifier) = magnified {
        loupe = if magnifier.sample_camera != 0 {
          layer
            .camera
            .as_ref()
            .and_then(|camera| camera.picture.as_ref())
            .map(|camera| (camera.texture.view.clone(), camera.texture.size))
        } else {
          Some((picture, layer.source.texture.size))
        }
        .map(|picture| (magnifier, picture));
      }
    }
    if let Some((magnifier, (picture, picture_size))) = loupe {
      state
        .loupe
        .draw(gpu, &view, size, magnifier, &picture, picture_size);
    }
    Ok(())
  }
}

/// Clears `view` to transparent: the layers are drawn over it, and where none
/// reaches the webview's backdrop shows through.
fn clear(gpu: &crate::gpu::Gpu, view: &wgpu::TextureView) {
  let mut encoder = gpu
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Screenwide workspace clear"),
    });
  encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some("Screenwide workspace clear"),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  });
  gpu.queue.submit([encoder.finish()]);
}
