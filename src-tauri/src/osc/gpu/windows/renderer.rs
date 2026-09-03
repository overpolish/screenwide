// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! CPU vertex builder and render constants for every Windows GPU OSC surface.
//! It mirrors the shared macOS OSC renderer. Geometry is pure math on top-left
//! pixel coordinates, so tool surfaces only provide their semantic scene.

use crate::osc::geometry::{Point, Rect, Size};
use crate::osc::style::{control_palette, ocr_palette, overlay_palette, ruler_palette};

pub(crate) const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/osc_gpu_vs.cso"));
pub(crate) const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/osc_gpu_ps.cso"));

/// One triangle-list vertex. `position` is already in NDC: the pixel-to-clip
/// mapping happens here so the vertex shader stays a pass-through.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vertex {
  pub position: [f32; 2],
  pub uv: [f32; 2],
  /// Pattern phase and edge length for boundary-aware marquee capsules.
  pub aux: [f32; 2],
  pub kind: u32,
  pub padding: u32,
}

const _: () = assert!(std::mem::size_of::<Vertex>() == 32);

/// Replaces Metal's nine fragment push-constant slots (b0-b8) with a single
/// cbuffer. Fields keep the Metal declaration order and every member is a
/// float4 row, which is what HLSL cbuffer packing gives us for free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderConstants {
  pub light_mode: [u32; 4],
  pub magnifier_box: [f32; 4],
  pub action_fills: [[f32; 4]; 2],
  pub control_colors: [[f32; 4]; 2],
  pub ocr_colors: [[f32; 4]; 8],
  pub overlay_shade: [f32; 4],
  pub ruler_colors: [[f32; 4]; 2],
  pub ruler_sample: [f32; 4],
  pub ruler_animation: [f32; 4],
  pub magnifier_source: [f32; 4],
  pub magnifier_sample: [f32; 4],
  pub magnifier_source_range: [f32; 4],
  pub magnifier_flags: [u32; 4],
  /// Appended for the OCR chrome: `.x` is the plate corner radius. macOS took
  /// its radius from the material surface's `cornerRadius` mask, which the
  /// folded-in chrome has no equivalent for.
  pub chrome: [f32; 4],
  pub chrome_outline: [f32; 4],
  /// Physical viewport size and source texel size for the Windows material
  /// blur sampled from the already-resident frozen desktop texture.
  pub chrome_backdrop: [f32; 4],
  /// The snapshot UV window after Ruler pan/zoom.
  pub chrome_source: [f32; 4],
  /// Halo sample radius for contrast-safe, CPU-rasterised text readouts.
  pub outlined_label: [f32; 4],
}

const _: () = assert!(std::mem::size_of::<RenderConstants>().is_multiple_of(16));

/// Lens edge length in points; the box is this times the display scale.
pub(crate) const MAGNIFIER_BOX_POINTS: f64 = 96.0;

impl RenderConstants {
  /// Mirrors `screenwide_region_osc_render_state`: palettes come from the
  /// platform-neutral Rust tokens, action fills stay zero because OCR controls
  /// re-push their own pair per draw call.
  pub(crate) fn new(light_mode: bool) -> Self {
    let controls = control_palette(light_mode);
    let ocr = ocr_palette(light_mode);
    let ruler = ruler_palette(light_mode);
    Self {
      light_mode: [u32::from(light_mode), 0, 0, 0],
      magnifier_box: [0.0; 4],
      action_fills: [[0.0; 4]; 2],
      control_colors: [controls.fill, controls.outline],
      ocr_colors: [
        ocr.primary_fill,
        ocr.primary_outline,
        ocr.qr_fill,
        ocr.qr_outline,
        ocr.error_fill,
        ocr.error_outline,
        ocr.selection_fill,
        ocr.selection_outline,
      ],
      overlay_shade: overlay_palette().shade,
      ruler_colors: [ruler.primary, ruler.info],
      ruler_sample: [0.0; 4],
      ruler_animation: [0.0; 4],
      magnifier_source: [0.0; 4],
      magnifier_sample: [0.0; 4],
      magnifier_source_range: [0.0, 0.0, 1.0, 1.0],
      magnifier_flags: [0; 4],
      chrome: [0.0; 4],
      chrome_outline: [0.0; 4],
      chrome_backdrop: [0.0; 4],
      chrome_source: [0.0, 0.0, 1.0, 1.0],
      outlined_label: [0.0; 4],
    }
  }

  /// Port of `screenwide_region_magnifier_make`. The box is sized and centred
  /// in physical pixels because the shader reads `SV_Position` directly, the
  /// way the Metal kernel read its thread position.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn set_magnifier(
    &mut self,
    point: Point,
    scale: f64,
    edges: u32,
    source: (u32, u32),
    sample: (f32, f32),
    source_min: (f32, f32),
    source_max: (f32, f32),
  ) {
    let size = ((MAGNIFIER_BOX_POINTS * scale).round() as i64).max(1);
    let x = (point.x * scale).round() as i64 - size / 2;
    let y = (point.y * scale).round() as i64 - size / 2;
    self.magnifier_box = [x as f32, y as f32, size as f32, size as f32];
    self.magnifier_source = [source.0 as f32, source.1 as f32, scale as f32, 0.0];
    self.magnifier_sample = [unit(sample.0), unit(sample.1), 0.0, 0.0];
    self.magnifier_source_range = [
      unit(source_min.0),
      unit(source_min.1),
      unit(source_max.0),
      unit(source_max.1),
    ];
    self.magnifier_flags = [edges, 1, 0, 0];
  }

  pub(crate) fn clear_magnifier(&mut self) {
    self.magnifier_box = [0.0; 4];
    self.magnifier_flags = [0; 4];
  }
}

fn unit(value: f32) -> f32 {
  value.clamp(0.0, 1.0)
}

/// Port of `screenwide_region_magnifier_anchor`: snaps the lens centre to the
/// dragged edge, clamped inside the frame.
pub(crate) fn magnifier_anchor(point: Point, frame: Rect, edges: u32) -> Point {
  let x = if edges & 1 != 0 {
    frame.origin.x
  } else if edges & 2 != 0 {
    frame.right()
  } else {
    point.x
  };
  let y = if edges & 4 != 0 {
    frame.origin.y
  } else if edges & 8 != 0 {
    frame.bottom()
  } else {
    point.y
  };
  Point {
    x: x.clamp(frame.origin.x, frame.right()),
    y: y.clamp(frame.origin.y, frame.bottom()),
  }
}

/// Hairline snapping: land the core on a pixel centre.
pub(crate) fn snap(value: f64, scale: f64) -> f64 {
  ((value * scale).floor() + 0.5) / scale
}

/// Handle centres snap to whole pixels instead, so their circles stay round.
pub(crate) fn snap_handle_center(value: f64, scale: f64) -> f64 {
  (value * scale).round() / scale
}

fn snap_handle_point(point: Point, scale: f64) -> Point {
  Point {
    x: snap_handle_center(point.x, scale),
    y: snap_handle_center(point.y, scale),
  }
}

fn ndc(view: Size, x: f64, y: f64) -> [f32; 2] {
  [
    (2.0 * x / view.width.max(1.0) - 1.0) as f32,
    (1.0 - 2.0 * y / view.height.max(1.0)) as f32,
  ]
}

fn push_quad(out: &mut Vec<Vertex>, corners: [[f32; 2]; 4], uvs: [[f32; 2]; 4], kind: u32) {
  push_quad_with_aux(out, corners, uvs, [0.0; 2], kind);
}

fn push_quad_with_aux(
  out: &mut Vec<Vertex>,
  corners: [[f32; 2]; 4],
  uvs: [[f32; 2]; 4],
  aux: [f32; 2],
  kind: u32,
) {
  let vertex = |index: usize| Vertex {
    position: corners[index],
    uv: uvs[index],
    aux,
    kind,
    padding: 0,
  };
  out.extend_from_slice(&[
    vertex(0),
    vertex(1),
    vertex(2),
    vertex(0),
    vertex(2),
    vertex(3),
  ]);
}

fn rect_corners(view: Size, rect: Rect) -> [[f32; 2]; 4] {
  [
    ndc(view, rect.origin.x, rect.origin.y),
    ndc(view, rect.right(), rect.origin.y),
    ndc(view, rect.right(), rect.bottom()),
    ndc(view, rect.origin.x, rect.bottom()),
  ]
}

const UNIT_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

fn is_empty(rect: Rect) -> bool {
  rect.size.width <= 0.0 || rect.size.height <= 0.0
}

pub(crate) fn add_quad(out: &mut Vec<Vertex>, view: Size, rect: Rect, kind: u32) {
  push_quad(out, rect_corners(view, rect), UNIT_UVS, kind);
}

pub(crate) fn add_texture_quad(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  texture_rect: Rect,
  kind: u32,
) {
  let min_u = texture_rect.origin.x as f32;
  let min_v = texture_rect.origin.y as f32;
  let max_u = texture_rect.right() as f32;
  let max_v = texture_rect.bottom() as f32;
  push_quad(
    out,
    rect_corners(view, rect),
    [
      [min_u, min_v],
      [max_u, min_v],
      [max_u, max_v],
      [min_u, max_v],
    ],
    kind,
  );
}

/// Both ends are extended by the half width so the fragment SDF can round the
/// caps without the quad clipping them.
pub(crate) fn add_line(
  out: &mut Vec<Vertex>,
  view: Size,
  start: Point,
  end: Point,
  width: f64,
  kind: u32,
) {
  let dx = end.x - start.x;
  let dy = end.y - start.y;
  let length = dx.hypot(dy);
  if length <= 0.0001 || width <= 0.0 {
    return;
  }
  let half = width * 0.5;
  let ux = dx / length;
  let uy = dy / length;
  let px = -uy * half;
  let py = ux * half;
  let extended_start = Point {
    x: start.x - ux * half,
    y: start.y - uy * half,
  };
  let extended_end = Point {
    x: end.x + ux * half,
    y: end.y + uy * half,
  };
  push_quad(
    out,
    [
      ndc(view, extended_start.x + px, extended_start.y + py),
      ndc(view, extended_end.x + px, extended_end.y + py),
      ndc(view, extended_end.x - px, extended_end.y - py),
      ndc(view, extended_start.x - px, extended_start.y - py),
    ],
    UNIT_UVS,
    kind,
  );
}

mod chrome;
mod ruler;
mod selection;

pub(crate) use chrome::{add_coverage_label, add_icon, add_label, add_outlined_label, add_plate};
pub(crate) use ruler::{add_ruler_arc, add_ruler_box};
pub(crate) use selection::{add_crop, add_crop_with_handles, add_selection};

/// The lens replaces Metal's compute pass, so it is a quad over
/// `RenderConstants::magnifier_box` and must be emitted last.
pub(crate) fn add_magnifier(out: &mut Vec<Vertex>, view: Size, constants: &RenderConstants) {
  let [x, y, width, height] = constants.magnifier_box;
  if constants.magnifier_flags[1] == 0 || width <= 0.0 || height <= 0.0 {
    return;
  }
  add_quad(
    out,
    view,
    Rect::from_xywh(
      f64::from(x),
      f64::from(y),
      f64::from(width),
      f64::from(height),
    ),
    45,
  );
}

#[cfg(test)]
#[path = "renderer/tests.rs"]
mod tests;
