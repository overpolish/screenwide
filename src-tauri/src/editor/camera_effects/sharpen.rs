// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-FileCopyrightText: 2021 Advanced Micro Devices, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later AND MIT

//! Enhance's sharpening for the stills `cpu.rs` draws: the twin of
//! `fsr_rcas` in `fsr1.wgsl`, AMD FidelityFX Super Resolution 1's RCAS,
//! whose MIT notice that file carries.

/// FSR's luma, twice the usual scale, which RCAS weighs its noise by.
fn luma([r, g, b]: [f32; 3]) -> f32 {
  b * 0.5 + (r * 0.5 + g)
}

/// The RGB of the 8-bit RGBA `rgba`, `width` by `height`, sharpened at
/// RCAS's full strength, one entry per pixel.
pub(super) fn sharpened(rgba: &[u8], (width, height): (u32, u32)) -> Vec<[u8; 3]> {
  let (width, height) = (width as usize, height as usize);
  let texel = |x: usize, y: usize| {
    let at = (y * width + x) * 4;
    [0, 1, 2].map(|channel| f32::from(rgba[at + channel]) / 255.0)
  };
  let mut out = Vec::with_capacity(width * height);
  for y in 0..height {
    for x in 0..width {
      let b = texel(x, y.saturating_sub(1));
      let d = texel(x.saturating_sub(1), y);
      let e = texel(x, y);
      let f = texel((x + 1).min(width - 1), y);
      let h = texel(x, (y + 1).min(height - 1));
      let mut lobe = f32::MIN;
      for channel in 0..3 {
        let lowest = b[channel].min(d[channel]).min(f[channel]).min(h[channel]);
        let highest = b[channel].max(d[channel]).max(f[channel]).max(h[channel]);
        let hit_min = lowest.min(e[channel]) / (4.0 * highest).max(1e-6);
        let hit_max = (1.0 - highest.max(e[channel])) / (4.0 * lowest - 4.0).min(-1e-6);
        lobe = lobe.max((-hit_min).max(hit_max));
      }
      let mut lobe = lobe.clamp(-(0.25 - 1.0 / 16.0), 0.0);
      // Where the cross looks like noise rather than detail, it holds back.
      let lumas = [b, d, e, f, h].map(luma);
      let spread = lumas.iter().copied().fold(f32::MIN, f32::max)
        - lumas.iter().copied().fold(f32::MAX, f32::min);
      let noise = ((0.25 * (lumas[0] + lumas[1] + lumas[3] + lumas[4]) - lumas[2]).abs()
        / spread.max(1e-6))
      .clamp(0.0, 1.0);
      lobe *= 1.0 - 0.5 * noise;
      out.push([0, 1, 2].map(|channel| {
        let value = (lobe * (b[channel] + d[channel] + f[channel] + h[channel]) + e[channel])
          / (4.0 * lobe + 1.0);
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
      }));
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::sharpened;

  #[test]
  fn an_edge_is_steepened_with_only_a_faint_overshoot() {
    // A soft step from dark to light: sharpened, its middle moves apart. The
    // lobe is capped, so the sides overshoot by a few levels at most rather
    // than ringing into a visible halo.
    let ramp = [40_u8, 40, 80, 160, 200, 200];
    let rgba: Vec<u8> = (0..3)
      .flat_map(|_| ramp.iter().flat_map(|&value| [value, value, value, 255]))
      .collect();
    let out = sharpened(&rgba, (6, 3));
    let row: Vec<u8> = out[6..12].iter().map(|pixel| pixel[0]).collect();
    assert!(row[2] < 80 && row[3] > 160, "{row:?}");
    assert!(
      row.iter().all(|&value| (24..=216).contains(&value)),
      "{row:?}"
    );
  }

  #[test]
  fn a_flat_picture_is_left_as_it_is() {
    let rgba = [90_u8, 120, 150, 255].repeat(16);
    assert!(sharpened(&rgba, (4, 4))
      .iter()
      .all(|&pixel| pixel == [90, 120, 150]));
  }
}
