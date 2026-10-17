// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing the retained scene into the workspace drawable.

use super::loupe::NativeMagnifier;
use super::{SceneState, WorkspaceScene};
use crate::editor::preview_platform::workspace_backdrop::BackdropStyle;
use crate::editor::preview_platform::CAPTIONS_LAYER_ID;
use screenwide_compositor::annotation_gpu::caption_overlay_typed;
use screenwide_compositor::compositor::{ComposedFrame, LayerDraw, LayerPlacement};

/// Where one layer's canvas is drawn in the workspace drawable, in its
/// pixels. The twin of `ScreenwideWorkspacePlacement`.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NativeWorkspacePlacement {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
}

/// The scene as last composed, at the revision, placements and backdrop it
/// was composed for, so a redraw of the selection chrome alone shows it
/// again. One composed with the crop magnifier has no revision: the loupe
/// follows the pointer, so it is never shown again.
pub(super) struct RetainedPicture {
  texture: wgpu::Texture,
  revision: Option<u64>,
  placements: Vec<NativeWorkspacePlacement>,
  backdrop: Option<BackdropStyle>,
}

impl RetainedPicture {
  fn shows(
    &self,
    revision: u64,
    size: (u32, u32),
    placements: &[NativeWorkspacePlacement],
    backdrop: BackdropStyle,
  ) -> bool {
    self.revision == Some(revision)
      && (self.texture.width(), self.texture.height()) == size
      && self.placements == placements
      && self.backdrop == Some(backdrop)
  }
}

impl WorkspaceScene {
  /// Whether the scene holds `count` layers, one for each placement a redraw
  /// would pass. A redraw that cannot draw must not take a drawable.
  #[cfg_attr(target_os = "windows", allow(dead_code))]
  pub(in crate::editor::preview_platform) fn ready(&self, count: usize) -> bool {
    self
      .read()
      .is_ok_and(|state| count > 0 && state.layers.len() == count)
  }

  /// Draws every layer into `target` at its placement, in drawable pixels,
  /// the first at the bottom, over the `backdrop` checkerboard, and the crop
  /// magnifier over them all. With `reuse`, a picture composed since the
  /// scene last changed, at the same placements, size and backdrop, is
  /// copied in instead of composing it again: the redraw is for the
  /// selection chrome alone. The magnifier follows the pointer, so it is
  /// always composed afresh.
  pub(in crate::editor::preview_platform) fn draw(
    &self,
    target: &wgpu::Texture,
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
    reuse: bool,
  ) -> Result<(), String> {
    let mut state = self.read()?;
    if state.layers.len() != placements.len() {
      return Err("The workspace placements do not match its layers".to_owned());
    }
    let gpu = state.compositor.gpu();
    let size = (target.width(), target.height());
    let magnified = magnifier.is_some_and(|magnifier| magnifier.active != 0);
    let revision = state.revision;
    let current = reuse
      && !magnified
      && state
        .picture
        .as_ref()
        .is_some_and(|picture| picture.shows(revision, size, placements, backdrop));
    if !current {
      let mut picture = match state.picture.take() {
        Some(picture) if (picture.texture.width(), picture.texture.height()) == size => picture,
        _ => RetainedPicture {
          texture: picture_texture(gpu, size),
          revision: None,
          placements: Vec::with_capacity(placements.len()),
          backdrop: None,
        },
      };
      compose(&state, &picture.texture, placements, magnifier, backdrop)?;
      picture.revision = (!magnified).then_some(revision);
      picture.placements.clear();
      picture.placements.extend_from_slice(placements);
      picture.backdrop = Some(backdrop);
      state.picture = Some(picture);
    }
    let picture = state
      .picture
      .as_ref()
      .ok_or_else(|| "The workspace picture is missing".to_owned())?;
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide workspace picture"),
      });
    encoder.copy_texture_to_texture(
      picture.texture.as_image_copy(),
      target.as_image_copy(),
      wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
    );
    gpu.queue.submit([encoder.finish()]);
    Ok(())
  }
}

/// A texture the size of the drawable to compose the scene into.
fn picture_texture(gpu: &screenwide_gpu::Gpu, (width, height): (u32, u32)) -> wgpu::Texture {
  gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide workspace picture"),
    size: wgpu::Extent3d {
      width,
      height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::Bgra8Unorm,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  })
}

/// Composes every layer of `state` into `target` at its placement over the
/// `backdrop` checkerboard, and the crop magnifier over them all.
fn compose(
  state: &SceneState,
  target: &wgpu::Texture,
  placements: &[NativeWorkspacePlacement],
  magnifier: Option<&NativeMagnifier>,
  backdrop: BackdropStyle,
) -> Result<(), String> {
  let gpu = state.compositor.gpu();
  let size = (target.width(), target.height());
  let view = target.create_view(&Default::default());
  state.backdrop.draw(gpu, &view, backdrop);
  let mut loupe = None;
  for (index, (layer, placement)) in state.layers.iter().zip(placements).enumerate() {
    if placement.width == 0 || placement.height == 0 {
      continue;
    }
    let geometry = &layer.geometry;
    let typing = state
      .typing
      .filter(|(typed, _, _)| *typed == layer.layer_id)
      .map(|(_, index, marks)| (index, marks));
    let mut prepared = layer
      .annotations
      .placed(geometry.placement, layer.hover, typing);
    if let Some((track, moment)) = &layer.captions {
      // The caption being typed into carries the caret and selection the
      // typing marks it with.
      let marks = state
        .typing
        .filter(|(typed, _, _)| *typed == CAPTIONS_LAYER_ID)
        .map(|(_, _, marks)| marks);
      prepared.draw_above(caption_overlay_typed(track, *moment, geometry.size, marks));
    }
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
          camera.drawn(
            &state.compositor,
            picture,
            (&state.effects, index, layer.seconds),
          )?,
          geometry,
          camera.drop_shadow,
          camera.on_top,
        ))
      })
      .transpose()?;
    // A camera pane is drawn with the camera's effects.
    let effected = if layer.source_is_camera {
      let matte = layer.source_matte.as_deref();
      state.effects.drawn(
        &state.compositor,
        (&layer.source.texture, layer.seconds),
        matte,
        index * 2,
      )?
    } else {
      None
    };
    // The bottom layer is drawn whole even when it is only a foreground:
    // there is nothing under it to lay it over.
    let picture = state.compositor.draw_layer(
      &view,
      effected.as_ref().unwrap_or(&layer.source.texture),
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
        // while it is drawn from; the effects' picture is drawn again each
        // time, and a decoded frame's is refilled from the next frame.
        retained_source: effected.is_none() && layer.source.retained,
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
