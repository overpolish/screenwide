// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where one draw's blur layers lie on the canvas, how finely, and the
//! canvas constants the passes that draw and read them take.

use super::super::constants::Constants;
use super::super::layer::CanvasBand;
use crate::annotation_gpu::PreviewArrow;
use screenwide_annotations::AnnotationKind;

/// How many layer texels one deviation of the blur spans.
const TEXELS_PER_DEVIATION: f32 = 2.0;

/// The longest side the layers are laid at, which only a blur narrower than
/// half a canvas pixel on a very large canvas reaches.
const LARGEST_SIDE: f32 = 4096.0;

/// The `annotation_blur` modes of the canvas pass that draws each layer.
const DRAWING_MARKS: f32 = 1.0;
const DRAWING_CURSOR: f32 = 3.0;

/// Where and how one draw's layers are laid.
#[derive(Clone, Copy)]
pub(in super::super) struct Plan {
  /// Layer texels per canvas pixel.
  scale: f32,
  /// Where the layers' first texel lies on the canvas, in canvas pixels.
  origin: [f32; 2],
  pub(super) size: (u32, u32),
  /// The deviation in layer texels.
  pub(super) sigma: f32,
  /// The spotlight whose shade the blurred annotations lie under, where any
  /// annotation does.
  pub(super) spotlight: Option<u32>,
  pub(super) cursor: bool,
}

impl Plan {
  /// The layers `values` call for over `arrows` on a canvas `size`, or
  /// `None` where no spotlight blurs or there is nothing for it to blur.
  /// They cover the whole canvas, or only as much of it round `part` as
  /// the blur reaches into it, where the draw holds only that part.
  pub(in super::super) fn of(
    values: &Constants,
    arrows: &[PreviewArrow],
    size: (u32, u32),
    part: Option<CanvasBand>,
  ) -> Option<Self> {
    let [_, _, deviation, strength] = values.cursor_blur;
    // A deviation that is not a number blurs nothing, as the shader reads it.
    let blurs = deviation.is_finite() && deviation >= 0.5;
    if !blurs || strength <= 0.0 || size.0 == 0 || size.1 == 0 {
      return None;
    }
    let below = (values.annotation_options[0] as usize).min(arrows.len());
    let total = (values.annotation_options[1] as usize).min(arrows.len());
    // While a scene crosses the order over, the blur layer follows the
    // nearer order, as the shader draws that layer.
    let camera_between = values.camera_effects[1] != 0.0 && values.camera_effects[3] >= 0.5;
    let topmost = |first: usize, last: usize| {
      (first..last)
        .rev()
        .find(|&index| arrows[index].kind == AnnotationKind::Spotlight.raw())
    };
    let spotlight = match topmost(below, total) {
      Some(index) => (index > 0 || camera_between).then_some(index),
      None => topmost(0, below).filter(|&index| index > 0),
    };
    let cursor = values.cursor_options[1] != 0;
    if spotlight.is_none() && !cursor {
      return None;
    }
    let [low, high] = covered(size, part, deviation);
    let extent = (high[0] - low[0]).max(high[1] - low[1]);
    let scale = (TEXELS_PER_DEVIATION / deviation).min(LARGEST_SIDE / extent);
    // Started on a whole texel of the grid the whole canvas's layers lie
    // on, so every part's layers sample the same points of it.
    let origin = low.map(|low| (low * scale).floor() / scale);
    let side = |axis: usize| (((high[axis] - origin[axis]) * scale).ceil() as u32).max(1);
    Some(Self {
      scale,
      origin,
      size: (side(0), side(1)),
      sigma: deviation * scale,
      spotlight: spotlight.map(|index| index as u32),
      cursor,
    })
  }

  fn spotlight_word(self) -> f32 {
    self.spotlight.map_or(-1.0, |index| index as f32)
  }

  /// The canvas pass's `annotation_blur` row while it reads the layers.
  pub(in super::super) fn reading(self) -> [f32; 4] {
    [
      self.scale,
      2.0,
      self.spotlight_word(),
      f32::from(u8::from(self.cursor)),
    ]
  }

  /// The canvas pass's `annotation_blur_origin` row.
  pub(in super::super) fn origin(self) -> [f32; 4] {
    [self.origin[0], self.origin[1], 0.0, 0.0]
  }

  /// `values` as the pass that draws the annotations' layer (`0`) or the
  /// cursor's (`1`) takes them: the part of the canvas the layers cover laid
  /// over them, every edge feathered over one of their texels.
  pub(in super::super) fn drawing(self, mut values: Constants, layer: usize) -> Constants {
    let mode = if layer == 0 {
      DRAWING_MARKS
    } else {
      DRAWING_CURSOR
    };
    values.placement = [
      -self.origin[0] * self.scale,
      -self.origin[1] * self.scale,
      1.0 / self.scale,
      1.0 / self.scale,
    ];
    values.motion[2] = 1.0 / self.scale;
    values.annotation_blur = [self.scale, mode, self.spotlight_word(), 0.0];
    // Nothing past the canvas's edge is in the layers.
    values.overflow = [0.0; 4];
    values
  }
}

/// How far the spotlights' blur of `deviation` canvas pixels reaches round
/// a pixel, in canvas pixels, with a texel either side for the layers'
/// filtered reads.
pub(in super::super) fn reach(deviation: f32) -> f32 {
  (deviation * 3.0).ceil() + 2.0
}

/// The corners, in canvas pixels, of what a canvas `size` drawn holding
/// `part` needs blurred by `deviation`: the whole canvas, or `part` and as
/// far round it as the blur reaches into it.
fn covered(size: (u32, u32), part: Option<CanvasBand>, deviation: f32) -> [[f32; 2]; 2] {
  let whole = [[0.0, 0.0], [size.0 as f32, size.1 as f32]];
  let Some(part) = part else {
    return whole;
  };
  let reach = reach(deviation);
  let low = [part.origin.0 as f32 - reach, part.origin.1 as f32 - reach];
  let high = [
    (part.origin.0 + part.size.0) as f32 + reach,
    (part.origin.1 + part.size.1) as f32 + reach,
  ];
  [
    [low[0].max(0.0), low[1].max(0.0)],
    [high[0].min(whole[1][0]), high[1].min(whole[1][1])],
  ]
}
