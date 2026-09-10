// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn maps(width: u32, height: u32) -> GradientMaps {
  GradientMaps {
    soft_edges: None,
    gx: vec![0; (width * height) as usize],
    gy: vec![0; (width * height) as usize],
    width,
    height,
  }
}

#[test]
fn probes_stop_at_sustained_edges_on_both_axes() {
  let mut maps = maps(12, 10);
  for y in 3..=5 {
    maps.gx[(y * maps.width + 2) as usize] = 30;
    maps.gx[(y * maps.width + 9) as usize] = 30;
  }
  for x in 5..=7 {
    maps.gy[(2 * maps.width + x) as usize] = 30;
    maps.gy[(8 * maps.width + x) as usize] = 30;
  }
  assert_eq!(
    ProbeIndex::new(&maps, 24).probes_at(6, 4),
    [
      PixelProbe {
        axis: ProbeAxis::Horizontal,
        start: 2,
        end: 9,
        position: 4,
      },
      PixelProbe {
        axis: ProbeAxis::Vertical,
        start: 2,
        end: 8,
        position: 6,
      },
    ]
  );
}

#[test]
fn isolated_speckle_does_not_stop_a_probe() {
  let mut maps = maps(10, 5);
  maps.gx[(2 * maps.width + 7) as usize] = 255;
  let horizontal = ProbeIndex::new(&maps, 24).probes_at(4, 2)[0];
  assert_eq!((horizontal.start, horizontal.end), (0, 9));
}

#[test]
fn a_bright_one_pixel_stroke_still_has_detectable_endpoints() {
  let (width, height) = (37, 26);
  let mut rgba = vec![51; width * height * 4];
  for pixel in rgba.chunks_exact_mut(4) {
    pixel[3] = 255;
  }
  for x in 14..=23 {
    let value = if x == 14 || x == 23 { 120 } else { 125 };
    let offset = (13 * width + x) * 4;
    rgba[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
  }
  let maps = super::super::analysis::compute_gradients(
    &rgba,
    width.try_into().unwrap(),
    height.try_into().unwrap(),
  );
  let horizontal = probes_at_threshold(&maps, 18, 13, 5)[0];
  assert!(horizontal.start >= 14 && horizontal.start <= 15);
  assert!(horizontal.end >= 23 && horizontal.end <= 24);
}

#[test]
fn split_antialiasing_mass_reaches_the_threshold() {
  let mut maps = maps(10, 5);
  for y in 1..=3 {
    maps.gx[(y * maps.width + 7) as usize] = 12;
    maps.gx[(y * maps.width + 8) as usize] = 12;
  }
  let horizontal = ProbeIndex::new(&maps, 18).probes_at(4, 2)[0];
  assert_eq!(horizontal.end, 7);
}

#[test]
fn cursor_local_scan_applies_sensitivity_without_rebuilding_the_index() {
  let mut maps = maps(12, 7);
  for y in 2..=4 {
    maps.gx[(y * maps.width + 3) as usize] = 7;
    maps.gx[(y * maps.width + 9) as usize] = 7;
  }
  let balanced = probes_at_threshold(&maps, 6, 3, 24)[0];
  assert_eq!((balanced.start, balanced.end), (0, 11));
  let subtle = probes_at_threshold(&maps, 6, 3, 5)[0];
  assert_eq!((subtle.start, subtle.end), (3, 9));
}

#[test]
fn indexed_lookup_uses_the_edge_under_the_pointer_only_as_the_start() {
  let mut maps = maps(10, 5);
  for y in 1..=3 {
    maps.gx[(y * maps.width + 4) as usize] = 30;
    maps.gx[(y * maps.width + 8) as usize] = 30;
  }
  let horizontal = ProbeIndex::new(&maps, 24).probes_at(4, 2)[0];
  assert_eq!((horizontal.start, horizontal.end), (4, 8));
}

#[test]
fn probes_find_the_peak_of_a_settled_soft_transition_on_both_axes() {
  let width = 40usize;
  let mut rgba = vec![255; width * width * 4];
  for y in 0..width {
    for x in 0..width {
      let inset = x.min(y);
      let level = match inset {
        0..=13 => 255,
        14 => 251,
        15 => 245,
        16 => 237,
        _ => 229,
      };
      let index = (y * width + x) * 4;
      rgba[index..index + 3].fill(level);
    }
  }
  let maps = super::super::analysis::compute_gradients(&rgba, width as u32, width as u32);
  let indexed = ProbeIndex::new(&maps, 24).probes_at(25, 25);
  let scanned = probes_at_threshold(&maps, 25, 25, 24);
  assert_eq!(indexed, scanned);
  assert_eq!(indexed[0].start, 16);
  assert_eq!(indexed[1].start, 16);
}
