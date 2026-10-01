// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A spotlight's prepared record, the distance that picks it and how much
//! light reaches a point.
//!
//! The record carries the box's corners in `a` and `b`, its corner radius in
//! `rounding` and how far its edge fades in `width`, all in the pixels it was
//! prepared in. [`light`] and [`shade`] are the arithmetic the Metal and WGSL
//! spotlight passes repeat per pixel; they live here so the rule has one
//! written form the tests hold the shaders to.

use super::model::share;
use crate::editor::annotations::geometry::ArrowGeometry;

/// The box between the two corners, in the space they are given in, rounded
/// by `radius` percent of its shorter side and fading over `softness` percent
/// of it.
pub(crate) fn prepare_spotlight(
  start: [f32; 2],
  end: [f32; 2],
  radius: f32,
  softness: f32,
) -> ArrowGeometry {
  let a = [start[0].min(end[0]), start[1].min(end[1])];
  let b = [start[0].max(end[0]), start[1].max(end[1])];
  let shortest = (b[0] - a[0]).min(b[1] - a[1]).max(0.0);
  let part = |percent: f32| shortest * share(f64::from(percent)) as f32 / 100.0;
  ArrowGeometry {
    a,
    b,
    rounding: part(radius),
    width: part(softness),
    ..ArrowGeometry::default()
  }
}

/// How far `point` falls from the prepared box: zero or less anywhere inside
/// it, so a press anywhere in the light picks it.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn spotlight_distance(point: [f32; 2], geometry: &ArrowGeometry) -> f32 {
  crate::editor::annotations::text::geometry::rounded_box_distance(
    point,
    geometry.a,
    geometry.b,
    geometry.rounding,
  )
}

/// How much of a spotlight's light reaches `point`: all of it well inside
/// the box, none outside it, and a smooth fall over its softness in from the
/// edge, so a soft spotlight never grows past the box its grips are on.
/// `feather` is how wide an antialiased edge is in these pixels, which is
/// all a spotlight with no softness fades over.
#[cfg(test)]
pub(crate) fn light(point: [f32; 2], geometry: &ArrowGeometry, feather: f32) -> f32 {
  let distance = spotlight_distance(point, geometry);
  let soft = geometry.width.max(0.0);
  let fall = ((distance + soft + feather * 0.5) / (soft + feather)).clamp(0.0, 1.0);
  1.0 - fall * fall * (3.0 - 2.0 * fall)
}

/// How much of the shade reaches a point, from every spotlight showing as
/// its presence - how far it has faded in - and the light it lets through
/// there. The shade is as strong as the most present spotlight, and each
/// spotlight lifts it by its own light as far as it is present, so two
/// spotlights light two places without darkening each other and one fading
/// in brightens its box as the shade around it deepens.
#[cfg(test)]
pub(crate) fn shade(spotlights: impl IntoIterator<Item = (f32, f32)>) -> f32 {
  let (layer, lit) =
    spotlights
      .into_iter()
      .fold((0.0_f32, 0.0_f32), |(layer, lit), (presence, light)| {
        let presence = presence.clamp(0.0, 1.0);
        (layer.max(presence), lit.max(presence * light))
      });
  (layer - lit).max(0.0)
}
