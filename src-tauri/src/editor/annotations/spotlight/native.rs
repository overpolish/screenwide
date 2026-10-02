// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A spotlight's retained draw record, and the blur pass it asks of the
//! source.
//!
//! `p0` and `p2` are the box's corners, placed like any point. `p1` is not a
//! point: it carries the corner radius and the softness, each as a percentage
//! of the box's shorter side, which no placement touches. `flags` carries
//! `BLUR` when what lies outside the light is blurred too.
//!
//! The blur is one pass over the whole source, applied with the redactions:
//! the source is softened everywhere a shade with blur reaches, as a blurred
//! redaction softens its box. Its record carries every spotlight's hole, four
//! points each in source pixels - the box's corners, its rounding and fade,
//! and how present it is beside the blur - so the pass lifts the blur
//! wherever light falls, by the same rule the shade is lifted by.

use super::model::share;
use crate::editor::annotations::flags::BLUR;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::native::NativeAnnotation;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::AnnotationKind;
use crate::editor::annotations::{AnnotationPoint, AnnotationStyle};

/// How wide the spotlights' blur is once whole, as a share of the source's
/// shorter side: a slight softening, enough to push what is outside the
/// light back, not enough to make it a smear.
const SPOTLIGHT_BLUR_SHARE: f32 = 1.0 / 250.0;

pub(crate) fn draw_points(
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
) -> [[f32; 2]; 3] {
  [
    [start.x as f32, start.y as f32],
    [share(style.radius) as f32, share(style.softness) as f32],
    [end.x as f32, end.y as f32],
  ]
}

/// The flags a spotlight's record carries.
pub(crate) fn flags(style: &AnnotationStyle) -> u32 {
  if style.blur {
    BLUR
  } else {
    0
  }
}

/// How present a record is this frame, from its reveal: a still, and a clip
/// past its fade, is wholly present.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn presence(item: &NativeAnnotation) -> f32 {
  if item.reveal.opacity.is_finite() {
    item.reveal.opacity.clamp(0.0, 1.0)
  } else {
    1.0
  }
}

/// How far a record's blur has arrived this frame: its presence, and of
/// that the share its reveal's `scale` carries, which is less than all of it
/// only while its light glides from or to a spotlight that does not blur.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn blur_presence(item: &NativeAnnotation) -> f32 {
  let share = if item.reveal.scale.is_finite() {
    item.reveal.scale.clamp(0.0, 1.0)
  } else {
    1.0
  };
  presence(item) * share
}

/// How far the spotlights' blur has arrived among `items`: as present as the
/// most present spotlight that blurs, and zero where none does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn blur_strength(items: &[NativeAnnotation]) -> f32 {
  items
    .iter()
    .filter(|item| {
      AnnotationKind::from_raw(item.kind) == Some(AnnotationKind::Spotlight)
        && item.flags & BLUR != 0
    })
    .map(blur_presence)
    .fold(0.0_f32, f32::max)
}

/// The blur pass among `items`, over a source `width` by `height` pixels:
/// how far the blur has arrived - as present as the most present spotlight
/// that blurs - and every spotlight's hole, four points each. A hole's
/// presence is measured against the blur's, so a spotlight arriving with its
/// own blur keeps its light clear throughout. `None` where no spotlight
/// showing blurs, and there is no pass to run.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn blur_holes(
  items: &[NativeAnnotation],
  width: u32,
  height: u32,
) -> Option<(f32, Vec<[f32; 2]>)> {
  let strength = blur_strength(items);
  if strength <= 0.0 || width == 0 || height == 0 {
    return None;
  }
  let holes = items
    .iter()
    .filter(|item| AnnotationKind::from_raw(item.kind) == Some(AnnotationKind::Spotlight))
    .flat_map(|item| {
      let geometry = super::geometry::prepare_spotlight(item.p0, item.p2, item.p1[0], item.p1[1]);
      [
        geometry.a,
        geometry.b,
        [geometry.rounding, geometry.width],
        [(presence(item) / strength).min(1.0), 0.0],
      ]
    })
    .collect();
  Some((strength, holes))
}

/// The standard deviation of the spotlights' blur over a source `width` by
/// `height` pixels, as it arrives with `strength`. It arrives by widening
/// from nothing, as a blurred redaction does: fading a finished blur in over
/// the sharp picture would lay the blurred copy over it as a glow.
pub(crate) fn blur_deviation(strength: f32, width: u32, height: u32) -> f32 {
  width.min(height) as f32 * SPOTLIGHT_BLUR_SHARE * strength.clamp(0.0, 1.0)
}
