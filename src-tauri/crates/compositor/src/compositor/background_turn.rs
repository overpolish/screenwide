// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The background giving way while a scene's blooms in over it, as the
//! canvas shader reads it: its own background rows beside the canvas's, then
//! where the bloom spreads from and how large each background is drawn.

use super::background::background_constants;
use super::*;

/// The twin of `BackgroundTurn` in `canvas_background.wgsl`; every member is
/// a 16-byte row.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TurnRows {
  solid_color: [f32; 4],
  base_color: [f32; 4],
  mesh_points: [[f32; 4]; 8],
  mesh_colors: [[f32; 4]; 4],
  /// The seed, whether it is a mesh, how many blobs it has, and whether it
  /// is a picture, which only the picture uploaded says.
  pub(super) options: [u32; 4],
  /// The generator and how many colours its palette reads.
  pub(super) style: [u32; 4],
  /// The classic mesh's warp, in percent, and the generator's speed.
  params: [f32; 4],
  /// Where the bloom spreads from, in canvas pixels, how far the farthest
  /// corner of the canvas is from there, and how far it has spread, eased.
  bloom: [f32; 4],
  /// How large the outgoing and the incoming backgrounds are drawn about the
  /// bloom's centre, then whether a background is giving way at all.
  pub(super) scale: [f32; 4],
}

impl TurnRows {
  pub(super) fn turning(&self) -> bool {
    self.scale[2] != 0.0
  }
}

/// The rows for `settings`' background turn on a canvas `size` pixels big,
/// and the path of the outgoing picture where it is one; empty rows where no
/// background is giving way.
pub(super) fn turn_rows(
  settings: &ScreenshotOutputSettings,
  size: (u32, u32),
) -> Result<(TurnRows, Option<&str>), String> {
  let Some(turn) = settings.background_turn.as_ref() else {
    return Ok((bytemuck::Zeroable::zeroed(), None));
  };
  let outgoing = &turn.outgoing;
  let ground = background_constants(outgoing.into())?;
  let picture = (outgoing.background_type == "image")
    .then_some(outgoing.background_image_path.as_deref())
    .flatten();
  let (width, height) = (f64::from(size.0), f64::from(size.1));
  let centre = (turn.centre.0 * width, turn.centre.1 * height);
  let reach = [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)]
    .map(|(x, y)| (x - centre.0).hypot(y - centre.1))
    .into_iter()
    .fold(1.0, f64::max);
  let (outgoing_scale, incoming_scale) = turn.scales();
  Ok((
    TurnRows {
      solid_color: colour_f32(&outgoing.background_color)?,
      base_color: ground.base,
      mesh_points: ground.points,
      mesh_colors: ground.colors,
      options: [
        outgoing.mesh_seed,
        u32::from(ground.mesh),
        outgoing.mesh_points.len() as u32,
        0,
      ],
      style: [ground.generator, ground.palette_size, 0, 0],
      params: [outgoing.mesh_warp_percent as f32, ground.speed, 0.0, 0.0],
      bloom: [
        centre.0 as f32,
        centre.1 as f32,
        reach as f32,
        turn.share.clamp(0.0, 1.0) as f32,
      ],
      scale: [outgoing_scale as f32, incoming_scale as f32, 1.0, 0.0],
    },
    picture,
  ))
}
