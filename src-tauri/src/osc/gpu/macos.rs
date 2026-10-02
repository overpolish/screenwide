// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The macOS OSC surfaces draw through the shared pipeline. Objective-C
//! builds the vertices, keeps the text and snapshot textures and presents;
//! this encodes the pass on the shared queue, so a command buffer the
//! surface commits afterwards to present runs after it.

mod renderer;

use std::ffi::c_void;

use super::pipeline::{RenderConstants, Vertex};
use crate::gpu::macos::{bgra_render_target, sampled_texture};
use crate::gpu::Gpu;
use renderer::RENDERER;

/// The twin of `ScreenwideRegionMagnifier`: the layer it reads, the anchor
/// and the shown part of that picture as shares of it, and the box in
/// drawable pixels.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeMagnifier {
  pub(crate) active: u32,
  pub(crate) pane_index: u32,
  pub(crate) layer_id: u32,
  pub(crate) sample_camera: u32,
  pub(crate) edges: u32,
  pub(crate) light_mode: u32,
  pub(crate) sample: [f32; 2],
  pub(crate) source_min: [f32; 2],
  pub(crate) source_max: [f32; 2],
  pub(crate) box_origin: [i32; 2],
  pub(crate) box_size: [u32; 2],
}

/// The twin of `ScreenwideRegionOscRenderState`, the palettes and animation
/// state a surface draws with.
#[repr(C)]
pub(crate) struct NativeRenderState {
  light_mode: u32,
  magnifier_box: [f32; 4],
  overlay_shade: [f32; 4],
  action_fills: [[f32; 4]; 2],
  control_fill: [f32; 4],
  control_outline: [f32; 4],
  ocr_colors: [[f32; 4]; 8],
  ruler_colors: [[f32; 4]; 2],
  ruler_sample: [f32; 4],
  ruler_animation: [f32; 4],
}

/// What a draw does with its magnifier: 0 nothing, 1 keep the lens's box
/// clear for a lens drawn before, or this, draw the lens itself last from
/// the magnifier source.
const LENS_DRAWN: u32 = 2;

/// The lens's own vertex kind in `osc.wgsl`.
const LENS_KIND: u32 = 49;

impl NativeRenderState {
  fn constants(&self) -> RenderConstants {
    RenderConstants {
      light_mode: [self.light_mode, 0, 0, 0],
      magnifier_box: self.magnifier_box,
      action_fills: self.action_fills,
      control_colors: [self.control_fill, self.control_outline],
      ocr_colors: self.ocr_colors,
      overlay_shade: self.overlay_shade,
      ruler_colors: self.ruler_colors,
      ruler_sample: self.ruler_sample,
      ruler_animation: self.ruler_animation,
      ..RenderConstants::new(self.light_mode != 0)
    }
  }
}

/// Draws `count` OSC vertices into `target`, a drawable's `id<MTLTexture>`,
/// clearing it first when `clear` is set. `label`, `secondary_label`,
/// `snapshot` and `magnifier_source` are `id<MTLTexture>`s or NULL; `lens`
/// says what `magnifier`, if any, does. Answers 0 when nothing was drawn.
///
/// # Safety
/// `target` and every texture given must be live textures of the shared
/// device; `vertices` must hold `count` vertices; `state` must be readable,
/// and `magnifier` NULL or readable.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_osc_draw(
  target: *mut c_void,
  clear: i32,
  vertices: *const Vertex,
  count: u32,
  state: *const NativeRenderState,
  magnifier: *const NativeMagnifier,
  lens: u32,
  label: *mut c_void,
  secondary_label: *mut c_void,
  snapshot: *mut c_void,
  magnifier_source: *mut c_void,
) -> i32 {
  let Some(state) = (unsafe { state.as_ref() }) else {
    return 0;
  };
  let vertices = if vertices.is_null() || count == 0 {
    &[][..]
  } else {
    unsafe { std::slice::from_raw_parts(vertices, count as usize) }
  };
  let magnifier = unsafe { magnifier.as_ref() }.filter(|magnifier| magnifier.active != 0);
  let textures = [label, secondary_label, snapshot, magnifier_source];
  let drawn = RENDERER
    .as_ref()
    .map_err(Clone::clone)
    .and_then(|renderer| {
      let renderer = renderer
        .lock()
        .map_err(|_| "The OSC renderer is poisoned".to_owned())?;
      let target = unsafe { bgra_render_target(renderer.gpu, target) }?;
      let mut constants = state.constants();
      let mut vertices = vertices.to_vec();
      let [label, secondary, snapshot, source] =
        textures.map(|raw| unsafe { shared_view(renderer.gpu, raw) });
      if let Some(magnifier) = magnifier.filter(|_| lens != 0) {
        place_lens(
          &mut constants,
          magnifier,
          source.as_ref().map(|(_, size)| *size),
        );
        if lens == LENS_DRAWN {
          vertices.extend(lens_quad(magnifier, (target.width(), target.height())));
        }
      }
      let view = |texture: &Option<(wgpu::TextureView, (u32, u32))>| {
        texture
          .as_ref()
          .map_or_else(|| renderer.placeholder.clone(), |(view, _)| view.clone())
      };
      renderer.draw(
        &target,
        clear != 0,
        &vertices,
        &constants,
        [
          view(&label),
          view(&secondary),
          view(&snapshot),
          view(&source),
        ],
      );
      Ok(())
    });
  match drawn {
    Ok(()) => 1,
    Err(error) => {
      eprintln!("The OSC could not be drawn: {error}");
      0
    }
  }
}

/// `raw` as a view to sample, with its size, or `None` for NULL or a texture
/// that cannot be shared.
///
/// # Safety
/// `raw` must be NULL or a live `id<MTLTexture>` of the shared device.
unsafe fn shared_view(gpu: &Gpu, raw: *mut c_void) -> Option<(wgpu::TextureView, (u32, u32))> {
  if raw.is_null() {
    return None;
  }
  let texture = unsafe { sampled_texture(gpu, raw) }
    .inspect_err(|error| eprintln!("An OSC texture could not be used: {error}"))
    .ok()?;
  Some((
    texture.create_view(&Default::default()),
    (texture.width(), texture.height()),
  ))
}

fn place_lens(
  constants: &mut RenderConstants,
  magnifier: &NativeMagnifier,
  source: Option<(u32, u32)>,
) {
  let [x, y] = magnifier.box_origin.map(|value| value as f32);
  let [width, height] = magnifier.box_size.map(|value| value as f32);
  constants.magnifier_box = [x, y, width, height];
  let (source_width, source_height) = source.unwrap_or((0, 0));
  constants.magnifier_source = [source_width as f32, source_height as f32, 0.0, 0.0];
  constants.magnifier_sample = [magnifier.sample[0], magnifier.sample[1], 0.0, 0.0];
  constants.magnifier_source_range = [
    magnifier.source_min[0],
    magnifier.source_min[1],
    magnifier.source_max[0],
    magnifier.source_max[1],
  ];
  constants.magnifier_flags = [magnifier.edges, 1, 0, 0];
}

/// A quad over the lens's box in a `size` target, which the lens kind fills.
fn lens_quad(magnifier: &NativeMagnifier, size: (u32, u32)) -> [Vertex; 6] {
  let ndc = |x: i64, y: i64| {
    [
      x as f32 / size.0.max(1) as f32 * 2.0 - 1.0,
      1.0 - y as f32 / size.1.max(1) as f32 * 2.0,
    ]
  };
  let [left, top] = magnifier.box_origin.map(i64::from);
  let right = left + i64::from(magnifier.box_size[0]);
  let bottom = top + i64::from(magnifier.box_size[1]);
  let vertex = |position| Vertex {
    position,
    uv: [0.0; 2],
    kind: LENS_KIND,
    padding: 0,
  };
  let [a, b, c, d] = [
    ndc(left, top),
    ndc(right, top),
    ndc(right, bottom),
    ndc(left, bottom),
  ];
  [a, b, c, a, c, d].map(vertex)
}
