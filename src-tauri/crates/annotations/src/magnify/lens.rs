// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Seeing through a lens, for the chrome: where what lies under a lens shows
//! in it, where a press inside it lands on what lies under it, and which
//! annotation a press takes hold of when lenses lie between it and the hand.
//!
//! A lens shows the point `q` under it at `centre + (q - centre) * zoom`, so
//! a press at `p` inside it lands on `centre + (p - centre) / zoom`. What lies
//! under its rim, past `1 / zoom` of the way out, shows nowhere: the lens
//! covers it and shows something nearer its middle there. Every space here
//! is one the lens is upright and unstretched in, as the chrome's display
//! points are on a layer standing upright.

/// One lens in the chrome's space: its box's centre and half size, its
/// corner radius and its zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lens {
  pub centre: [f64; 2],
  pub half: [f64; 2],
  pub radius: f64,
  pub zoom: f64,
}

impl Lens {
  /// The lens over the box from `low` to `high`, rounded by `percent` of its
  /// shorter side, enlarging `zoom` times.
  pub fn from_box(low: [f64; 2], high: [f64; 2], percent: f64, zoom: f64) -> Self {
    let half = [
      (high[0] - low[0]).abs() / 2.0,
      (high[1] - low[1]).abs() / 2.0,
    ];
    let share = if percent.is_finite() {
      percent.clamp(0.0, 50.0) / 100.0
    } else {
      0.0
    };
    Self {
      centre: [(low[0] + high[0]) / 2.0, (low[1] + high[1]) / 2.0],
      half,
      radius: 2.0 * half[0].min(half[1]) * share,
      zoom,
    }
  }

  /// Whether `point` lies on the lens.
  pub fn covers(&self, point: [f64; 2]) -> bool {
    if !(self.half[0] > 0.0 && self.half[1] > 0.0) {
      return false;
    }
    let rounding = self.radius.min(self.half[0]).min(self.half[1]).max(0.0);
    let q = [
      (point[0] - self.centre[0]).abs() - (self.half[0] - rounding),
      (point[1] - self.centre[1]).abs() - (self.half[1] - rounding),
    ];
    let outside = q[0].max(0.0).hypot(q[1].max(0.0));
    outside + q[0].max(q[1]).min(0.0) - rounding <= 0.0
  }

  /// The view through this lens alone.
  pub fn view(&self) -> LensView {
    LensView::IDENTITY.through(self)
  }
}

/// How a run of lenses carries a point: what lies under them at `p` is
/// `p * scale + offset`, one uniform scale about the lenses' middles, so a box
/// seen through them is still a box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensView {
  pub scale: f64,
  pub offset: [f64; 2],
}

impl LensView {
  /// No lens at all.
  pub const IDENTITY: Self = Self {
    scale: 1.0,
    offset: [0.0, 0.0],
  };

  /// This view, then `lens` under it: a press read through this view lands
  /// on `lens`, and is read on through it.
  pub fn through(self, lens: &Lens) -> Self {
    let zoom = if lens.zoom.is_finite() && lens.zoom > 0.0 {
      lens.zoom
    } else {
      1.0
    };
    let keep = 1.0 - 1.0 / zoom;
    Self {
      scale: self.scale / zoom,
      offset: [
        self.offset[0] / zoom + lens.centre[0] * keep,
        self.offset[1] / zoom + lens.centre[1] * keep,
      ],
    }
  }

  /// What lies under the lenses at `point`.
  pub fn beneath(&self, point: [f64; 2]) -> [f64; 2] {
    [
      point[0] * self.scale + self.offset[0],
      point[1] * self.scale + self.offset[1],
    ]
  }

  /// Where the lenses show `point`, which lies under them.
  pub fn seen(&self, point: [f64; 2]) -> [f64; 2] {
    [
      (point[0] - self.offset[0]) / self.scale,
      (point[1] - self.offset[1]) / self.scale,
    ]
  }

  /// Whether the view goes through any lens.
  pub fn is_identity(&self) -> bool {
    *self == Self::IDENTITY
  }
}

/// Where `point`, on an annotation, shows, and the view a press there reads
/// back through: through every lens in `above` covering where it shows by
/// then, bottom first, or `None` where a lens covers it and shows something
/// else there. `above` holds the lenses over the annotation on its picture,
/// in the same space as `point`.
pub fn shown_at(
  point: [f64; 2],
  above: impl IntoIterator<Item = Lens>,
) -> Option<([f64; 2], LensView)> {
  let mut at = point;
  let mut view = LensView::IDENTITY;
  for lens in above {
    if !lens.covers(at) {
      continue;
    }
    let shown = lens.view().seen(at);
    if !lens.covers(shown) {
      return None;
    }
    at = shown;
    view = lens.view().then_under(view);
  }
  Some((at, view))
}

impl LensView {
  /// `self`, the view through a lens on top, read on through `under`, the
  /// view through the lenses below it.
  fn then_under(self, under: Self) -> Self {
    Self {
      scale: self.scale * under.scale,
      offset: [
        self.offset[0] * under.scale + under.offset[0],
        self.offset[1] * under.scale + under.offset[1],
      ],
    }
  }
}

/// What a press takes hold of among annotations stacked first at the bottom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensPick {
  /// The annotation taken hold of.
  pub index: usize,
  /// How the press reaches it: through the lenses over it, or directly.
  pub view: LensView,
}

/// The topmost annotation a press takes hold of, seeing through lenses.
/// `point_on(index)` is the press as it lands on that annotation's own
/// picture. Going down from the top, an annotation `hits` at the press is
/// taken; a lens covering it is not taken at once: the press is read
/// through it and the search carries on below it, among what is drawn
/// there, and only where nothing drawn is hit is the lens itself taken, to
/// be moved. An inside seen through a lens - a shape's, a stroke's - never
/// takes the press from the lens, or a lens over a large shape could not be
/// moved. Off every lens, `hits` is asked first of what is drawn (`false`)
/// and then of insides (`true`), as with no lens at all. `lens` says which
/// annotations are lenses, and `under(lens, index)` whether the annotation
/// at `index` lies under that lens, on its picture.
pub fn pick_through(
  count: usize,
  point_on: impl Fn(usize) -> [f64; 2],
  lens: impl Fn(usize) -> Option<Lens>,
  under: impl Fn(usize, usize) -> bool,
  hits: impl Fn(usize, [f64; 2], bool) -> bool,
) -> Option<LensPick> {
  let search = Search {
    point_on: &point_on,
    lens: &lens,
    under: &under,
    hits: &hits,
  };
  search
    .pass(count, None, false)
    .or_else(|| search.pass(count, None, true))
}

/// Inside a lens: which one, the press as it lands on its picture, and the
/// view through every lens the search has gone through so far.
#[derive(Clone, Copy)]
struct Within {
  lens: usize,
  point: [f64; 2],
  view: LensView,
}

struct Search<'a, P, L, U, H> {
  point_on: &'a P,
  lens: &'a L,
  under: &'a U,
  hits: &'a H,
}

impl<P, L, U, H> Search<'_, P, L, U, H>
where
  P: Fn(usize) -> [f64; 2],
  L: Fn(usize) -> Option<Lens>,
  U: Fn(usize, usize) -> bool,
  H: Fn(usize, [f64; 2], bool) -> bool,
{
  /// What is taken among the annotations under `below`, or inside a lens
  /// only among those under it, read through it: what is drawn, or with
  /// `insides`, what is grabbed by its inside.
  fn pass(&self, below: usize, within: Option<Within>, insides: bool) -> Option<LensPick> {
    for index in (0..below).rev() {
      if within.is_some_and(|over| !(self.under)(over.lens, index)) {
        continue;
      }
      let (point, view) = within.map_or(((self.point_on)(index), LensView::IDENTITY), |over| {
        (over.point, over.view)
      });
      let at = view.beneath(point);
      if let Some(found) = (self.lens)(index).filter(|found| found.covers(at)) {
        // The drawn pass took the lens already wherever it covers the press.
        if insides {
          return None;
        }
        let deeper = Within {
          lens: index,
          point,
          view: view.through(&found),
        };
        return self
          .pass(index, Some(deeper), false)
          .or(Some(LensPick { index, view }));
      }
      if (self.hits)(index, at, insides) {
        return Some(LensPick { index, view });
      }
    }
    None
  }
}

#[cfg(test)]
#[path = "lens_tests.rs"]
mod tests;
