// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// One decoded frame's luma at the tracking size, rows packed without
/// padding.
#[derive(Clone, Debug)]
pub(crate) struct LumaFrame {
  /// The frame's presentation time, in source milliseconds.
  pub(crate) ms: u64,
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) pixels: Vec<u8>,
}

/// One pyramid level as floats, which is what the flow's bilinear samples
/// and gradients are taken in.
#[derive(Clone, Debug)]
pub(crate) struct Plane {
  pub(crate) width: usize,
  pub(crate) height: usize,
  pub(crate) data: Vec<f32>,
}

impl Plane {
  /// The value at `(x, y)`, clamped to the plane's edge.
  #[inline]
  pub(crate) fn at_clamped(&self, x: isize, y: isize) -> f32 {
    let x = x.clamp(0, self.width as isize - 1) as usize;
    let y = y.clamp(0, self.height as isize - 1) as usize;
    self.data[y * self.width + x]
  }

  /// Half the size, each pixel the mean of the four it covers. The decoder
  /// has already filtered the tracking size down from the source, so a box
  /// is enough to keep the coarse levels from aliasing.
  fn halved(&self) -> Self {
    let width = (self.width / 2).max(1);
    let height = (self.height / 2).max(1);
    let mut data = vec![0.0; width * height];
    for y in 0..height {
      let top = &self.data[(2 * y).min(self.height - 1) * self.width..][..self.width];
      let bottom = &self.data[(2 * y + 1).min(self.height - 1) * self.width..][..self.width];
      let row = &mut data[y * width..][..width];
      for (x, value) in row.iter_mut().enumerate() {
        let left = 2 * x;
        let right = (left + 1).min(self.width - 1);
        *value = 0.25 * (top[left] + top[right] + bottom[left] + bottom[right]);
      }
    }
    Self {
      width,
      height,
      data,
    }
  }
}

/// How many levels a pyramid has. Four halvings let a 15 pixel window follow
/// a movement of about a hundred tracking pixels a frame, which is a fast
/// flick of a scroll at the tracking size.
pub(crate) const LEVELS: usize = 4;

/// A frame and its halvings, finest first.
#[derive(Clone, Debug)]
pub(crate) struct Pyramid {
  pub(crate) ms: u64,
  pub(crate) levels: Vec<Plane>,
}

impl Pyramid {
  pub(crate) fn new(frame: &LumaFrame) -> Self {
    let base = Plane {
      width: frame.width as usize,
      height: frame.height as usize,
      data: frame.pixels.iter().map(|&value| f32::from(value)).collect(),
    };
    let mut levels = Vec::with_capacity(LEVELS);
    levels.push(base);
    while levels.len() < LEVELS {
      let next = levels[levels.len() - 1].halved();
      levels.push(next);
    }
    Self {
      ms: frame.ms,
      levels,
    }
  }

  pub(crate) fn width(&self) -> f32 {
    self.levels[0].width as f32
  }

  pub(crate) fn height(&self) -> f32 {
    self.levels[0].height as f32
  }
}
