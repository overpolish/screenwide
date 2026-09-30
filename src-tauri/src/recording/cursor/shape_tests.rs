// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::recognise;
use crate::recording::cursor::format::CursorStyle;

/// A drawn cursor: `#` is solid, `H` solid and the hotspot, anything else
/// clear. Each row is one pixel.
fn drawn(rows: &[&str]) -> CursorStyle {
  let width = rows[0].len();
  let mut hotspot = (0.0, 0.0);
  let alpha: Vec<u8> = rows
    .iter()
    .enumerate()
    .flat_map(|(y, row)| row.chars().enumerate().map(move |(x, pixel)| (x, y, pixel)))
    .map(|(x, y, pixel)| {
      if pixel == 'H' {
        hotspot = (x as f64 + 0.5, y as f64 + 0.5);
      }
      if matches!(pixel, '#' | 'H') {
        255
      } else {
        0
      }
    })
    .collect();
  recognise(width, rows.len(), &alpha, hotspot)
}

fn turned(rows: &[&str]) -> Vec<String> {
  (0..rows[0].len())
    .map(|x| rows.iter().map(|row| row.as_bytes()[x] as char).collect())
    .collect()
}

/// The classic I-beam most text views still draw, 9x18: flat serifs top and
/// bottom, and a short bar across the stem at the hotspot.
const CLASSIC_I_BEAM: [&str; 18] = [
  "####.####",
  "#########",
  "#########",
  "..#####..",
  "...###...",
  "...###...",
  "...###...",
  "...###...",
  "..#####..",
  "..##H##..",
  "..#####..",
  "...###...",
  "...###...",
  "...###...",
  "..#####..",
  "#########",
  "#########",
  "####.####",
];

#[test]
fn an_i_beam_is_recognised_whatever_design_draws_it() {
  assert_eq!(drawn(&CLASSIC_I_BEAM), CursorStyle::IBeam);
  // Twice as big, with its serifs curled off the stem, as the newer design
  // draws it.
  let newer = [
    "..##########..",
    "##############",
    "######..######",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....##H#.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    ".....####.....",
    "######..######",
    "##############",
    "..##########..",
  ];
  assert_eq!(drawn(&newer), CursorStyle::IBeam);
  let lying = turned(&CLASSIC_I_BEAM);
  let lying: Vec<&str> = lying.iter().map(String::as_str).collect();
  assert_eq!(drawn(&lying), CursorStyle::VerticalIBeam);
}

#[test]
fn an_arrow_is_recognised_by_its_tip_and_straight_edge() {
  let arrow = [
    "H..........",
    "##.........",
    "###........",
    "####.......",
    "#####......",
    "######.....",
    "#######....",
    "########...",
    "#########..",
    "##########.",
    "###########",
    "######.....",
    "###.###....",
    "##..###....",
    "#....###...",
    ".....###...",
    "......##...",
  ];
  assert_eq!(drawn(&arrow), CursorStyle::Arrow);
}

#[test]
fn a_pointing_hand_is_recognised_by_its_raised_finger() {
  let hand = [
    ".....H..........",
    "....###.........",
    "....###.........",
    "....###.........",
    "....######......",
    "....#########...",
    ".##.##########..",
    "###############.",
    "################",
    ".###############",
    "..##############",
    "..##############",
    "...############.",
    "....##########..",
    ".....########...",
    ".....########...",
  ];
  assert_eq!(drawn(&hand), CursorStyle::PointingHand);
}

#[test]
fn a_double_headed_arrow_is_a_resize_either_way() {
  let upright = [
    "....#....",
    "...###...",
    "..#####..",
    ".#######.",
    "#########",
    "...###...",
    "...###...",
    "...#H#...",
    "...###...",
    "...###...",
    "...###...",
    "#########",
    ".#######.",
    "..#####..",
    "...###...",
    "....#....",
  ];
  assert_eq!(drawn(&upright), CursorStyle::ResizeVertical);
  let lying = turned(&upright);
  let lying: Vec<&str> = lying.iter().map(String::as_str).collect();
  assert_eq!(drawn(&lying), CursorStyle::ResizeHorizontal);
}

/// A 15x15 picture from `solid`, which is asked of each pixel's centre as an
/// offset from the middle, with the hotspot in the middle.
fn round(solid: impl Fn(f64, f64) -> bool) -> CursorStyle {
  let alpha: Vec<u8> = (0..15)
    .flat_map(|y| (0..15).map(move |x| (x as f64 - 7.0, y as f64 - 7.0)))
    .map(|(x, y)| if solid(x, y) { 255 } else { 0 })
    .collect();
  recognise(15, 15, &alpha, (7.5, 7.5))
}

#[test]
fn a_crosshair_and_a_struck_ring_are_told_apart() {
  assert_eq!(round(|x, y| x == 0.0 || y == 0.0), CursorStyle::Crosshair);
  let struck = round(|x, y| {
    let distance = x.hypot(y);
    (5.6..=7.2).contains(&distance) || ((x - y).abs() <= 1.0 && distance <= 6.0)
  });
  assert_eq!(struck, CursorStyle::NotAllowed);
}

#[test]
fn a_shape_no_rule_describes_stays_custom() {
  // A disc, a box and a dot: nothing about them says which cursor they mean.
  assert_eq!(round(|x, y| x.hypot(y) <= 7.0), CursorStyle::Custom);
  assert_eq!(round(|_, _| true), CursorStyle::Custom);
  assert_eq!(round(|x, y| x.hypot(y) <= 1.0), CursorStyle::Custom);
  assert_eq!(recognise(4, 4, &[0; 16], (2.0, 2.0)), CursorStyle::Custom);
}
