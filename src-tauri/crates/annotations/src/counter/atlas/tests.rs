// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A cell as wide as its digits and as tall as its radius, like a text engine
/// would measure one.
fn measure(texts: &[String]) -> impl FnMut(usize, f32) -> Option<(u32, u32)> + '_ {
  |index, radius| {
    Some((
      texts[index].len() as u32 * 10 + 2,
      (radius * 2.0) as u32 + 2,
    ))
  }
}

fn numbers(texts: &[String], radius: f32) -> Vec<CounterNumber<'_>> {
  texts
    .iter()
    .map(|text| CounterNumber {
      text: text.as_bytes(),
      radius,
      style: 0,
    })
    .collect()
}

fn run(
  atlas: &mut CounterAtlas,
  texts: &[String],
  radius: f32,
) -> (Vec<AtlasRect>, Vec<AtlasDraw>, AtlasLayout) {
  let mut rects = vec![AtlasRect::default(); texts.len()];
  let mut draws = Vec::new();
  let layout = atlas.frame(
    &numbers(texts, radius),
    measure(texts),
    &mut rects,
    &mut draws,
  );
  (rects, draws, layout)
}

fn texts(range: std::ops::Range<u32>) -> Vec<String> {
  range.map(|value| value.to_string()).collect()
}

fn overlaps(a: &AtlasRect, b: &AtlasRect) -> bool {
  a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

#[test]
fn a_repeated_frame_rasterises_nothing_and_keeps_its_cells() {
  let mut atlas = CounterAtlas::default();
  let texts = texts(1..20);
  let (first, drawn, layout) = run(&mut atlas, &texts, 20.0);
  assert!(layout.fresh);
  assert_eq!(drawn.len(), texts.len());
  let (second, drawn, layout) = run(&mut atlas, &texts, 20.0);
  assert!(!layout.fresh);
  assert!(drawn.is_empty());
  assert_eq!(first, second);
}

#[test]
fn a_new_size_is_added_beside_the_cells_already_drawn() {
  let mut atlas = CounterAtlas::default();
  let texts = texts(1..4);
  let (first, _, _) = run(&mut atlas, &texts, 20.0);
  let (grown, drawn, layout) = run(&mut atlas, &texts, 21.0);
  assert!(!layout.fresh);
  assert_eq!(drawn.len(), texts.len());
  for rect in &grown {
    assert!(first.iter().all(|old| !overlaps(old, rect)));
  }
}

/// A counter numbered 12 and a text box reading "12" at the same size are set
/// differently, so they are two cells rather than one drawn the wrong way.
#[test]
fn the_same_text_set_two_ways_takes_two_cells() {
  let mut atlas = CounterAtlas::default();
  let texts = vec!["12".to_owned(), "12".to_owned()];
  let mut numbers = numbers(&texts, 20.0);
  numbers[1].style = 1;
  let mut rects = vec![AtlasRect::default(); 2];
  let mut draws = Vec::new();
  atlas.frame(&numbers, measure(&texts), &mut rects, &mut draws);
  assert_ne!(rects[0], rects[1]);
  assert_eq!(draws.len(), 2);
}

#[test]
fn the_same_number_at_the_same_size_shares_one_cell() {
  let mut atlas = CounterAtlas::default();
  let texts = vec!["7".to_owned(), "7".to_owned(), "8".to_owned()];
  let (rects, drawn, _) = run(&mut atlas, &texts, 20.0);
  assert_eq!(rects[0], rects[1]);
  assert_ne!(rects[0], rects[2]);
  assert_eq!(drawn.len(), 2);
}

#[test]
fn a_number_too_small_or_empty_is_not_drawn() {
  let mut atlas = CounterAtlas::default();
  let texts = vec![String::new(), "3".to_owned()];
  let (rects, drawn, _) = run(&mut atlas, &texts, 0.5);
  assert!(drawn.is_empty());
  assert!(rects.iter().all(|rect| *rect == AtlasRect::default()));
}

/// Once a frame's new numbers no longer fit, only that frame's numbers are
/// kept: the atlas sizes itself to what is on screen, not to everything it
/// has ever drawn.
#[test]
fn a_full_atlas_keeps_only_the_frame_that_overflowed_it() {
  let mut atlas = CounterAtlas::default();
  let early = texts(0..50);
  run(&mut atlas, &early, 20.0);
  let mut radius = 20.0;
  let layout = loop {
    radius += 0.25;
    let (_, _, layout) = run(&mut atlas, &early[..1], radius);
    if layout.fresh {
      break layout;
    }
  };
  assert!(layout.size.0 * layout.size.1 <= BASE_WIDTH * BASE_HEIGHT * 2);
  let (_, drawn, _) = run(&mut atlas, &early[1..], 20.0);
  assert!(!drawn.is_empty());
}

/// Thousands of counters on screen at once each get their own cell, inside
/// the atlas and clear of every other.
#[test]
fn thousands_of_numbers_all_find_a_place() {
  let mut atlas = CounterAtlas::default();
  let texts = texts(0..3_000);
  let (rects, drawn, layout) = run(&mut atlas, &texts, 24.0);
  assert_eq!(drawn.len(), texts.len());
  for (index, rect) in rects.iter().enumerate() {
    assert!(rect.width > 0.0);
    assert!(rect.x + rect.width <= layout.size.0 as f32);
    assert!(rect.y + rect.height <= layout.size.1 as f32);
    assert!(rects[index + 1..]
      .iter()
      .all(|other| !overlaps(rect, other)));
  }
}
