// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a recolouring highlight inks the text under it: each glyph's edge
//! keeps its share of the glyph, every glyph is inked as fully as the
//! brightest on its line, and overlapping highlights merge.

use super::*;

/// An encoded channel in linear light.
fn light(channel: u8) -> f64 {
  let encoded = f64::from(channel) / 255.0;
  if encoded <= 0.04045 {
    encoded / 12.92
  } else {
    ((encoded + 0.055) / 1.055).powf(2.4)
  }
}

fn light_rgb(rgb: [u8; 3]) -> [f64; 3] {
  rgb.map(light)
}

fn light_luminance(rgb: [u8; 3]) -> f64 {
  let [r, g, b] = light_rgb(rgb);
  0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// The edge coverages laid just left of each word, nearest the page first.
const RAMP: [f64; 3] = [0.25, 0.5, 0.75];

/// `page`, with a soft left edge on every word: three columns covered by
/// `RAMP`'s shares of `ink`, mixed in light the way type is drawn.
fn soft_page(surface: [u8; 3], ink: [u8; 3]) -> CapturedImage {
  let mut source = page(surface, ink);
  let encode = |light: f64| {
    let encoded = if light <= 0.003_130_8 {
      light * 12.92
    } else {
      1.055 * light.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
  };
  for y in 0..HEIGHT {
    for x in (3..WIDTH).filter(|x| is_text(*x, y) && !is_text(x - 1, y)) {
      for (step, cover) in RAMP.iter().enumerate() {
        let column = x - 3 + step as u32;
        let index = ((y * WIDTH + column) * 4) as usize;
        let mixed: [u8; 3] = std::array::from_fn(|channel| {
          encode(light(surface[channel]) * (1.0 - cover) + light(ink[channel]) * cover)
        });
        source.rgba[index..index + 3].copy_from_slice(&mixed);
      }
    }
  }
  source
}

/// The highlight over light text on a dark page, as the selection reads it.
fn over_dark(source: &CapturedImage) -> CapturedImage {
  composed(
    source,
    vec![highlight(
      HighlightTone {
        surface: 0.1,
        ink: 0.92,
      },
      false,
    )],
  )
}

#[test]
fn a_recoloured_edge_keeps_the_share_of_light_it_had() {
  // Light text turned dark keeps its weight only if each edge pixel keeps the
  // share of the glyph's light it had. A curve steeper than a straight line,
  // or one measured in encoded values, swells the glyph once it is dark.
  let source = soft_page([24, 26, 30], [235, 235, 235]);
  let image = over_dark(&source);
  let (page, text) = (at(&image, 64, 50), at(&image, 50, 50));
  for (step, cover) in RAMP.iter().enumerate() {
    let edge = at(&image, 45 + step as u32, 50);
    let share = (light_luminance(page) - light_luminance(edge))
      / (light_luminance(page) - light_luminance(text));
    // The floor that keeps faint page texture off takes a sliver of each.
    let expected = (cover - 0.04) / 0.96;
    assert!(
      (share - expected).abs() < 0.05,
      "{cover}: {share} {page:?} {edge:?} {text:?}"
    );
  }
}

#[test]
fn a_recoloured_edge_is_inked_in_its_glyphs_colour_not_the_pages() {
  // Cyan text on a dark page: every edge pixel is the highlight mixed with
  // the glyph's own ink, not with a greyer ink taken from the page it held.
  let source = soft_page([24, 26, 30], [70, 200, 210]);
  let image = over_dark(&source);
  let (page, text) = (light_rgb(at(&image, 64, 50)), light_rgb(at(&image, 50, 50)));
  for step in 0..RAMP.len() as u32 {
    let edge = light_rgb(at(&image, 45 + step, 50));
    let shares: Vec<f64> = (0..3)
      .filter(|channel| (page[*channel] - text[*channel]).abs() > 0.05)
      .map(|channel| (page[channel] - edge[channel]) / (page[channel] - text[channel]))
      .collect();
    let (low, high) = shares
      .iter()
      .fold((f64::MAX, f64::MIN), |(low, high), share| {
        (low.min(*share), high.max(*share))
      });
    assert!(
      high - low < 0.02,
      "{step}: {shares:?} {page:?} {edge:?} {text:?}"
    );
  }
}

#[test]
fn dim_and_coloured_text_is_inked_as_fully_as_the_brightest_on_its_line() {
  // A terminal line in three colours, the selection having read its ink from
  // the brightest. Measured against that alone, purple and dim grey would
  // pass for the faint edges of white text and all but vanish.
  const WHITE: [u8; 3] = [235, 235, 235];
  const PURPLE: [u8; 3] = [120, 90, 240];
  const GREY: [u8; 3] = [110, 110, 110];
  let mut source = page([24, 26, 30], WHITE);
  for y in 0..HEIGHT {
    for x in (0..WIDTH).filter(|x| is_text(*x, y)) {
      let index = ((y * WIDTH + x) * 4) as usize;
      let colour = [GREY, WHITE, PURPLE][(x / 24 % 3) as usize];
      source.rgba[index..index + 3].copy_from_slice(&colour);
    }
  }
  let image = over_dark(&source);
  // The middle of a word of each colour.
  for x in [56, 80, 104] {
    let core = at(&image, x, 50);
    assert!(luminance(core) < 0.25, "{x}: {core:?}");
  }
}

#[test]
fn overlapping_highlights_merge_rather_than_invert() {
  // The second reads the page as it was before the first recoloured it, not
  // the first's yellow, which over a dark page would pass for bright ink.
  let source = page([24, 26, 30], [235, 235, 235]);
  let tone = HighlightTone {
    surface: 0.1,
    ink: 0.92,
  };
  let once = composed(&source, vec![highlight(tone, false)]);
  let twice = composed(
    &source,
    vec![highlight(tone, false), highlight(tone, false)],
  );
  for (x, y) in [(64, 50), (50, 50), (64, 100)] {
    assert!(
      near(at(&twice, x, y), at(&once, x, y), 2),
      "{x},{y}: {:?} {:?}",
      at(&twice, x, y),
      at(&once, x, y)
    );
  }
}

#[test]
fn a_label_the_line_is_drawn_on_is_tinted_with_its_type() {
  // A terminal's red label with black type on it, the line's height and
  // wider than any stroke. Taken for one big glyph, it would be inked dark
  // and its type, at the page's brightness, turned the highlight's yellow.
  // As a fill it is tinted like a marker over an image: still red, with its
  // type still darker than it.
  let mut source = page([24, 26, 30], [235, 235, 235]);
  // Set a word's width apart from the white words, so only the label's own
  // type is near it.
  for y in 30..70 {
    for x in 76..224 {
      let label = (100..200).contains(&x);
      let lettered = label && (44..56).contains(&y) && (104..196).contains(&x) && (x - 104) % 8 < 4;
      let colour = match (label, lettered) {
        (false, _) => [24, 26, 30],
        (true, true) => [10, 10, 10],
        (true, false) => [155, 0, 0],
      };
      let index = ((y * WIDTH + x) * 4) as usize;
      source.rgba[index..index + 3].copy_from_slice(&colour);
    }
  }
  let image = composed(
    &source,
    vec![highlight(
      HighlightTone {
        surface: 0.1,
        ink: 0.4,
      },
      false,
    )],
  );
  let (label, letter) = (at(&image, 150, 50), at(&image, 152, 50));
  assert!(u16::from(label[0]) > u16::from(label[1]) + 60, "{label:?}");
  assert!(
    luminance(letter) + 0.05 < luminance(label),
    "{letter:?} {label:?}"
  );
}
