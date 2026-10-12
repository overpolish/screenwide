// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// An axis-aligned box in tracking pixels, `x1` and `y1` exclusive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
  pub(crate) x0: f32,
  pub(crate) y0: f32,
  pub(crate) x1: f32,
  pub(crate) y1: f32,
}

impl Rect {
  pub(crate) fn width(&self) -> f32 {
    (self.x1 - self.x0).max(0.0)
  }

  pub(crate) fn height(&self) -> f32 {
    (self.y1 - self.y0).max(0.0)
  }

  pub(crate) fn area(&self) -> f32 {
    self.width() * self.height()
  }

  pub(crate) fn centre(&self) -> [f32; 2] {
    [0.5 * (self.x0 + self.x1), 0.5 * (self.y0 + self.y1)]
  }

  pub(crate) fn intersect(&self, other: &Self) -> Self {
    Self {
      x0: self.x0.max(other.x0),
      y0: self.y0.max(other.y0),
      x1: self.x1.min(other.x1),
      y1: self.y1.min(other.y1),
    }
  }

  /// Grown by `factor` about its centre.
  pub(crate) fn scaled(&self, factor: f32) -> Self {
    let [cx, cy] = self.centre();
    let (hw, hh) = (0.5 * self.width() * factor, 0.5 * self.height() * factor);
    Self {
      x0: cx - hw,
      y0: cy - hh,
      x1: cx + hw,
      y1: cy + hh,
    }
  }

  pub(crate) fn contains(&self, [x, y]: [f32; 2]) -> bool {
    x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
  }
}

/// A movement, uniform change of size and rotation: `x' = a x - b y + tx`,
/// `y' = b x + a y + ty`. Rotation is fitted so a turning object's points
/// still agree, and ignored by everything that draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Similarity {
  pub(crate) a: f32,
  pub(crate) b: f32,
  pub(crate) tx: f32,
  pub(crate) ty: f32,
}

impl Similarity {
  pub(crate) const IDENTITY: Self = Self {
    a: 1.0,
    b: 0.0,
    tx: 0.0,
    ty: 0.0,
  };

  pub(crate) fn apply(&self, [x, y]: [f32; 2]) -> [f32; 2] {
    [
      self.a * x - self.b * y + self.tx,
      self.b * x + self.a * y + self.ty,
    ]
  }

  /// `self` followed by `next`.
  pub(crate) fn then(&self, next: &Self) -> Self {
    Self {
      a: next.a * self.a - next.b * self.b,
      b: next.a * self.b + next.b * self.a,
      tx: next.a * self.tx - next.b * self.ty + next.tx,
      ty: next.b * self.tx + next.a * self.ty + next.ty,
    }
  }

  pub(crate) fn scale(&self) -> f32 {
    (self.a * self.a + self.b * self.b).sqrt()
  }

  /// The same change of size and rotation, moved by `[dx, dy]`.
  pub(crate) fn shifted(&self, [dx, dy]: [f32; 2]) -> Self {
    Self {
      tx: self.tx + dx,
      ty: self.ty + dy,
      ..*self
    }
  }

  /// The box around `rect`'s corners once transformed.
  pub(crate) fn apply_rect(&self, rect: &Rect) -> Rect {
    let corners = [
      self.apply([rect.x0, rect.y0]),
      self.apply([rect.x1, rect.y0]),
      self.apply([rect.x0, rect.y1]),
      self.apply([rect.x1, rect.y1]),
    ];
    let mut out = Rect {
      x0: f32::INFINITY,
      y0: f32::INFINITY,
      x1: f32::NEG_INFINITY,
      y1: f32::NEG_INFINITY,
    };
    for [x, y] in corners {
      out.x0 = out.x0.min(x);
      out.y0 = out.y0.min(y);
      out.x1 = out.x1.max(x);
      out.y1 = out.y1.max(y);
    }
    out
  }
}

pub(crate) fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
  ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// A small deterministic generator, so the same clip always tracks to the
/// same path.
pub(crate) struct Rng(u32);

impl Rng {
  pub(crate) fn new(seed: u32) -> Self {
    Self(seed.max(1))
  }

  pub(crate) fn below(&mut self, limit: usize) -> usize {
    let mut x = self.0;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.0 = x;
    (x as usize) % limit.max(1)
  }
}
