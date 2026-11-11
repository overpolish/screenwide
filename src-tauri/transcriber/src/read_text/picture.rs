// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The picture text is read in, and the sizes both models see it at. It is
//! resized as OpenCV's bilinear resize does, which the models were made with:
//! each pixel taken from the four around its centre, with no averaging over
//! more when it shrinks.

/// A picture of red, green, blue and alpha bytes, row by row from the top.
pub(super) struct Picture<'a> {
  pub rgba: &'a [u8],
  pub width: usize,
  pub height: usize,
}

/// One axis of a resize: for each pixel out, the two pixels in it lies
/// between and how far towards the second.
fn axis(start: usize, length: usize, size: usize) -> Vec<(usize, usize, f32)> {
  let scale = length as f32 / size as f32;
  (0..size)
    .map(|out| {
      let at = (out as f32 + 0.5) * scale - 0.5;
      let (near, towards) = if at <= 0.0 {
        (0, 0.0)
      } else if at >= (length - 1) as f32 {
        (length - 1, 0.0)
      } else {
        (at.floor() as usize, at - at.floor())
      };
      (start + near, start + (near + 1).min(length - 1), towards)
    })
    .collect()
}

impl Picture<'_> {
  /// The region `[left, top, width, height]` resized to `size`, handing
  /// `put` each pixel out with its blue, green and red, 0 to 255: the order
  /// the models take their channels in.
  pub(super) fn resample(
    &self,
    [left, top, width, height]: [usize; 4],
    (out_width, out_height): (usize, usize),
    mut put: impl FnMut(usize, usize, [f32; 3]),
  ) {
    let columns = axis(left, width, out_width);
    let rows = axis(top, height, out_height);
    let channel = |x: usize, y: usize, c: usize| f32::from(self.rgba[(y * self.width + x) * 4 + c]);
    for (out_y, &(y0, y1, down)) in rows.iter().enumerate() {
      for (out_x, &(x0, x1, across)) in columns.iter().enumerate() {
        let mix = |c: usize| {
          let top = channel(x0, y0, c) * (1.0 - across) + channel(x1, y0, c) * across;
          let bottom = channel(x0, y1, c) * (1.0 - across) + channel(x1, y1, c) * across;
          top * (1.0 - down) + bottom * down
        };
        put(out_x, out_y, [mix(2), mix(1), mix(0)]);
      }
    }
  }
}
