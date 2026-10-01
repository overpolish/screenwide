// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;
const LIGHT: [u8; 3] = [250, 250, 248];
const DARK: [u8; 3] = [24, 26, 30];

/// A word: where it starts and how wide it is.
type Word = (u32, u32);

/// Three lines of words, 20 pixels of ink tall on a 36 pixel pitch. The
/// middle line reaches further right than the others.
const LINES: [(u32, &[Word]); 3] = [
  (40, &[(20, 60), (90, 50), (150, 70)]),
  (76, &[(20, 40), (70, 90), (170, 80), (260, 40)]),
  (112, &[(20, 70), (100, 60)]),
];
const INK_HEIGHT: u32 = 20;

fn page(surface: [u8; 3], ink: [u8; 3], scale: u32) -> Vec<u8> {
  let (width, height) = (WIDTH * scale, HEIGHT * scale);
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    for x in 0..width {
      let (sx, sy) = (x / scale, y / scale);
      let inked = LINES.iter().any(|(top, words)| {
        (*top..top + INK_HEIGHT).contains(&sy)
          && words
            .iter()
            .any(|(left, wide)| (*left..left + wide).contains(&sx) && sx % 7 != 3)
      });
      let [r, g, b] = if inked { ink } else { surface };
      rgba.extend_from_slice(&[r, g, b, 255]);
    }
  }
  rgba
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn select_on(rgba: &[u8], scale: u32, start: AnnotationPoint, end: AnnotationPoint) -> Selection {
  let pixels = HighlightPixels {
    rgba,
    width: WIDTH * scale,
    height: HEIGHT * scale,
  };
  let factor = f64::from(scale);
  select(Some((pixels, (factor, factor))), start, end, 24.0)
}

/// The band's ends and middle, rounded, which is what a reader judges a
/// highlight by.
fn span(band: &HighlightBand) -> (i64, i64, i64) {
  (
    band.left.round() as i64,
    band.right.round() as i64,
    ((band.top + band.bottom) * 0.5).round() as i64,
  )
}

#[test]
fn a_selection_across_lines_covers_them_the_way_selecting_text_does() {
  let rgba = page(LIGHT, DARK, 1);
  // Pressed well inside the first line's second word and let go well inside
  // the last line's first word.
  let selection = select_on(&rgba, 1, point(110.0, 50.0), point(40.0, 120.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 3, "{spans:?}");
  // The first line from where it was pressed to its own end.
  assert_eq!(spans[0].0, 110, "{spans:?}");
  assert!((220..=224).contains(&spans[0].1), "{spans:?}");
  // The line between in full, to its own further end.
  assert!((16..=20).contains(&spans[1].0), "{spans:?}");
  assert!((300..=304).contains(&spans[1].1), "{spans:?}");
  // The last line from its start to where it was let go.
  assert!((16..=20).contains(&spans[2].0), "{spans:?}");
  assert_eq!(spans[2].1, 40, "{spans:?}");
  // Each band sits on its own line.
  assert_eq!([spans[0].2, spans[1].2, spans[2].2], [50, 86, 122]);
}

#[test]
fn a_selection_dragged_upwards_covers_the_same_lines() {
  let rgba = page(LIGHT, DARK, 1);
  let down = select_on(&rgba, 1, point(110.0, 50.0), point(40.0, 120.0));
  let up = select_on(&rgba, 1, point(40.0, 120.0), point(110.0, 50.0));
  assert_eq!(down, up);
}

#[test]
fn a_selection_within_one_line_cuts_words_where_it_was_pressed_and_let_go() {
  let rgba = page(LIGHT, DARK, 1);
  let selection = select_on(&rgba, 1, point(80.0, 84.0), point(190.0, 88.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 1, "{spans:?}");
  assert_eq!((spans[0].0, spans[0].1), (80, 190), "{spans:?}");
}

#[test]
fn an_end_let_go_near_a_words_edge_settles_on_it() {
  let rgba = page(LIGHT, DARK, 1);
  // Pressed three pixels into a word and let go three short of another's end.
  let selection = select_on(&rgba, 1, point(73.0, 84.0), point(246.0, 88.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 1, "{spans:?}");
  assert!((66..=70).contains(&spans[0].0), "{spans:?}");
  assert!((250..=254).contains(&spans[0].1), "{spans:?}");
}

#[test]
fn a_selection_let_go_past_the_end_of_its_line_stops_at_its_last_word() {
  let rgba = page(LIGHT, DARK, 1);
  let selection = select_on(&rgba, 1, point(80.0, 84.0), point(390.0, 86.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 1, "{spans:?}");
  assert!((300..=304).contains(&spans[0].1), "{spans:?}");
}

#[test]
fn the_page_and_its_ink_are_read_whichever_way_round_they_are() {
  let light = select_on(
    &page(LIGHT, DARK, 1),
    1,
    point(30.0, 50.0),
    point(200.0, 50.0),
  );
  assert!(light.tone.surface > 0.9, "{:?}", light.tone);
  assert!(light.tone.ink < 0.2, "{:?}", light.tone);
  // Light text on a dark page is the case an opacity highlight fails on;
  // the recolouring only needs to know which end the page is at.
  let dark = select_on(
    &page(DARK, LIGHT, 1),
    1,
    point(30.0, 50.0),
    point(200.0, 50.0),
  );
  assert!(dark.tone.surface < 0.15, "{:?}", dark.tone);
  assert!(dark.tone.ink > 0.8, "{:?}", dark.tone);
  assert_eq!(light.bands, dark.bands);
}

#[test]
fn a_picture_at_twice_the_source_reads_the_same_bands_in_source_pixels() {
  let once = select_on(
    &page(LIGHT, DARK, 1),
    1,
    point(110.0, 50.0),
    point(40.0, 120.0),
  );
  let twice = select_on(
    &page(LIGHT, DARK, 2),
    2,
    point(110.0, 50.0),
    point(40.0, 120.0),
  );
  assert_eq!(once.bands.len(), twice.bands.len());
  for (a, b) in once.bands.iter().zip(&twice.bands) {
    for (x, y) in [
      (a.left, b.left),
      (a.right, b.right),
      (a.top, b.top),
      (a.bottom, b.bottom),
    ] {
      assert!((x - y).abs() <= 1.5, "{a:?} {b:?}");
    }
  }
}

#[test]
fn a_selection_over_no_text_is_one_band_along_the_drag() {
  let rgba = page(LIGHT, DARK, 1);
  let selection = select_on(&rgba, 1, point(40.0, 170.0), point(300.0, 175.0));
  assert_eq!(
    selection.bands,
    vec![HighlightBand {
      left: 40.0,
      top: 158.0,
      right: 300.0,
      bottom: 182.0
    }]
  );
  // Nothing stands out from the page, so the ink is taken to lie at the far
  // end from it.
  assert!(selection.tone.surface > 0.9 && selection.tone.ink < 0.8);
}

#[test]
fn without_a_picture_the_selection_is_one_band_along_the_drag() {
  let selection = select(None, point(300.0, 60.0), point(100.0, 90.0), 20.0);
  assert_eq!(
    selection.bands,
    vec![HighlightBand {
      left: 100.0,
      top: 50.0,
      right: 300.0,
      bottom: 70.0
    }]
  );
  assert_eq!(selection.tone, HighlightTone::default());
}

/// A picture of any size to lay scenes out on, read at one source pixel a
/// pixel.
struct Picture {
  width: u32,
  height: u32,
  rgba: Vec<u8>,
}

impl Picture {
  fn new(width: u32, height: u32, surface: [u8; 3]) -> Self {
    Self {
      width,
      height,
      rgba: [surface[0], surface[1], surface[2], 255].repeat((width * height) as usize),
    }
  }

  fn paint(&mut self, x: u32, y: u32, colour: [u8; 3]) {
    let at = ((y * self.width + x) * 4) as usize;
    self.rgba[at..at + 3].copy_from_slice(&colour);
  }

  fn fill(&mut self, left: u32, top: u32, wide: u32, tall: u32, colour: [u8; 3]) {
    for y in top..top + tall {
      for x in left..left + wide {
        self.paint(x, y, colour);
      }
    }
  }

  /// A line of words drawn the way `page` draws them.
  fn words(&mut self, top: u32, words: &[Word], ink: [u8; 3]) {
    for y in top..top + INK_HEIGHT {
      for (left, wide) in words {
        for x in (*left..left + wide).filter(|x| x % 7 != 3) {
          self.paint(x, y, ink);
        }
      }
    }
  }

  fn select(&self, start: AnnotationPoint, end: AnnotationPoint) -> Selection {
    let pixels = HighlightPixels {
      rgba: &self.rgba,
      width: self.width,
      height: self.height,
    };
    select(Some((pixels, (1.0, 1.0))), start, end, 24.0)
  }
}

#[test]
fn a_picture_beside_two_lines_neither_joins_them_nor_is_covered() {
  // An avatar on its own tile just left of a name and the line under it, the
  // way a review or a comment lays them out.
  let mut picture = Picture::new(WIDTH, HEIGHT, DARK);
  picture.fill(30, 36, 56, 60, [120, 110, 90]);
  picture.fill(44, 50, 24, 30, [150, 40, 40]);
  picture.words(40, &[(96, 60), (166, 70)], LIGHT);
  picture.words(70, &[(96, 40), (146, 50)], LIGHT);
  let selection = picture.select(point(110.0, 50.0), point(180.0, 80.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 2, "{spans:?}");
  assert_eq!([spans[0].2, spans[1].2], [50, 80], "{spans:?}");
  for band in &selection.bands {
    assert!(band.left > 88.0, "{band:?}");
    assert!(band.bottom - band.top < 30.0, "{band:?}");
  }
}

#[test]
fn a_pane_beside_the_page_ends_its_lines() {
  // A page of text with another window's pane beside it, in a colour of its
  // own and with text of its own.
  let mut picture = Picture::new(WIDTH, HEIGHT, LIGHT);
  picture.fill(312, 0, 88, HEIGHT, [40, 44, 52]);
  for (top, words) in LINES {
    picture.words(top, words, DARK);
    picture.words(top, &[(324, 60)], LIGHT);
  }
  let selection = picture.select(point(30.0, 50.0), point(290.0, 120.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 3, "{spans:?}");
  assert_eq!([spans[0].2, spans[1].2, spans[2].2], [50, 86, 122]);
  for band in &selection.bands {
    assert!(band.right < 312.0, "{band:?}");
  }
}

#[test]
fn a_rule_between_two_columns_is_not_a_letter_joining_them() {
  // Two columns of a table, a one-pixel rule between them: a letter's width
  // across, but running down past every line.
  let mut picture = Picture::new(WIDTH, HEIGHT, LIGHT);
  picture.fill(190, 0, 1, HEIGHT, DARK);
  for top in [40, 76, 112] {
    picture.words(top, &[(20, 70), (100, 80)], DARK);
    picture.words(top, &[(200, 60), (270, 50)], DARK);
  }
  let selection = picture.select(point(30.0, 50.0), point(170.0, 120.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 3, "{spans:?}");
  for band in &selection.bands {
    assert!(band.right < 190.0, "{band:?}");
  }
}

/// Three rows of two cells each, a column's width apart, the way a table of
/// properties lays out: `left` words in the first cell and `right` in the
/// second.
fn two_cells(left: &[Word], right: &[Word]) -> Picture {
  let mut picture = Picture::new(WIDTH, HEIGHT, LIGHT);
  for top in [40, 76, 112] {
    picture.words(top, left, DARK);
    picture.words(top, right, DARK);
  }
  picture
}

#[test]
fn a_selection_across_a_table_covers_each_row_across_its_cells() {
  let picture = two_cells(&[(20, 60)], &[(200, 70)]);
  // Pressed at the start of the first row's first cell, let go at the end of
  // the last row's second.
  let selection = picture.select(point(22.0, 50.0), point(266.0, 122.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 3, "{spans:?}");
  for band in &selection.bands {
    assert!(band.left < 20.0 && band.right > 270.0, "{band:?}");
  }
}

#[test]
fn a_selection_down_one_column_keeps_to_it() {
  let picture = two_cells(&[(20, 60), (90, 60)], &[(220, 70), (300, 60)]);
  let selection = picture.select(point(30.0, 50.0), point(120.0, 122.0));
  let spans: Vec<_> = selection.bands.iter().map(span).collect();
  assert_eq!(spans.len(), 3, "{spans:?}");
  for band in &selection.bands {
    assert!(band.right < 170.0, "{band:?}");
  }
}

/// Colour that changes every few pixels, the way a photo does: no colour is
/// most of it, so there is no page to read text from.
fn photo() -> Picture {
  let mut picture = Picture::new(WIDTH, HEIGHT, LIGHT);
  for y in 0..HEIGHT {
    for x in 0..WIDTH {
      let cell = (x / 3) * 7_919 + (y / 3) * 104_729;
      let noise = |salt: u32| (cell.wrapping_mul(2_654_435_761).rotate_left(salt) >> 24) as u8;
      picture.paint(x, y, [noise(3), noise(11), noise(19)]);
    }
  }
  picture
}

#[test]
fn a_selection_over_a_photo_tints_one_band_along_the_drag() {
  let selection = photo().select(point(40.0, 60.0), point(300.0, 64.0));
  assert_eq!(
    selection.bands,
    vec![HighlightBand {
      left: 40.0,
      top: 48.0,
      right: 300.0,
      bottom: 72.0
    }]
  );
  assert_eq!(selection.tone, HighlightTone::UNREAD);
}

fn box_tone_of(
  picture: &Picture,
  start: AnnotationPoint,
  bands: &[HighlightBand],
) -> HighlightTone {
  let pixels = HighlightPixels {
    rgba: &picture.rgba,
    width: picture.width,
    height: picture.height,
  };
  box_tone(Some((pixels, (1.0, 1.0))), start, bands, 24.0)
}

#[test]
fn a_box_over_a_dark_page_reads_its_light_text() {
  // A terminal: light text on a dark page, and a box laid over two lines.
  let mut picture = Picture::new(WIDTH, HEIGHT, DARK);
  picture.words(40, &[(20, 60), (90, 50)], LIGHT);
  picture.words(76, &[(20, 40), (70, 90)], LIGHT);
  let bands = [HighlightBand {
    left: 10.0,
    top: 36.0,
    right: 180.0,
    bottom: 100.0,
  }];
  let tone = box_tone_of(&picture, point(10.0, 36.0), &bands);
  assert!(tone.surface < 0.15, "{tone:?}");
  assert!(tone.ink > 0.8, "{tone:?}");
}

#[test]
fn a_box_over_a_photo_has_no_page_to_recolour() {
  let bands = [HighlightBand {
    left: 40.0,
    top: 40.0,
    right: 300.0,
    bottom: 120.0,
  }];
  let tone = box_tone_of(&photo(), point(40.0, 40.0), &bands);
  assert_eq!(tone, HighlightTone::UNREAD);
}

#[test]
fn a_selection_pressed_on_a_label_runs_on_across_its_line() {
  // A terminal's error label - dark type on red, taller than its line and
  // most of what lies around the press - starting the first of three lines,
  // the drag pressed on it. Read around the press alone, the label passed for
  // the page, the terminal around it for a pane, and the first line ended
  // where the label did.
  let mut picture = Picture::new(WIDTH, HEIGHT, DARK);
  picture.fill(0, 30, 200, 40, [152, 0, 0]);
  picture.words(40, &[(24, 40), (72, 50), (130, 50)], DARK);
  picture.words(40, &[(230, 50), (290, 60)], LIGHT);
  picture.words(76, &[(20, 90), (120, 80)], LIGHT);
  picture.words(112, &[(20, 70), (100, 60)], LIGHT);
  // Let go past the end of the last line, as a drag down a terminal is. With
  // the smaller marker the label is taller than a solid run, as on a Retina
  // capture, and the gaps between its letters are solid columns.
  for marker in [24.0, 12.0] {
    let pixels = HighlightPixels {
      rgba: &picture.rgba,
      width: picture.width,
      height: picture.height,
    };
    let selection = select(
      Some((pixels, (1.0, 1.0))),
      point(40.0, 50.0),
      point(380.0, 122.0),
      marker,
    );
    let spans: Vec<_> = selection.bands.iter().map(span).collect();
    assert_eq!(spans.len(), 3, "{marker}: {spans:?}");
    // The label is part of its line, and the line runs on past it to its end.
    let first = selection.bands[0];
    assert!(
      first.left < 50.0 && first.right >= 350.0,
      "{marker}: {first:?}"
    );
    assert_eq!(
      selection.tone.surface,
      luma(DARK.map(|c| f64::from(c) / 255.0))
    );
  }
}

#[test]
fn a_label_at_the_top_runs_on_past_a_cursor_above_and_stems_below() {
  // A label on the picture's first line, a cursor block over its corner at
  // the very top, and the next line's tall stems below the label's edge,
  // past a gap. Neither runs on past the line as a pane does: the cursor is
  // all the picture leaves above, and the stems are another line's.
  let mut picture = Picture::new(WIDTH, HEIGHT, DARK);
  picture.fill(40, 0, 16, 6, LIGHT);
  picture.fill(0, 4, 200, 40, [152, 0, 0]);
  picture.words(14, &[(24, 40), (72, 50), (130, 50)], DARK);
  picture.words(14, &[(230, 50), (290, 60)], LIGHT);
  picture.fill(190, 46, 8, 40, LIGHT);
  picture.words(56, &[(20, 90), (120, 60)], LIGHT);
  picture.words(96, &[(20, 70), (100, 60)], LIGHT);
  let pixels = HighlightPixels {
    rgba: &picture.rgba,
    width: picture.width,
    height: picture.height,
  };
  let selection = select(
    Some((pixels, (1.0, 1.0))),
    point(30.0, 24.0),
    point(380.0, 106.0),
    12.0,
  );
  let first = selection.bands[0];
  assert!(first.right >= 350.0, "{:?}", selection.bands);
}

#[test]
fn a_cursor_resting_on_a_label_does_not_end_its_line() {
  // A terminal's cursor block sits on an error label's top edge, below the
  // line before it. Its columns run on past the label upwards, but only a
  // cell wide: a pane beside the text is at least a line's height across.
  let mut picture = Picture::new(WIDTH, HEIGHT, DARK);
  picture.words(10, &[(20, 90), (120, 60)], LIGHT);
  picture.fill(184, 34, 12, 30, LIGHT);
  picture.fill(0, 64, 200, 40, [152, 0, 0]);
  picture.words(74, &[(24, 40), (72, 50), (130, 50)], DARK);
  picture.words(74, &[(230, 50), (290, 60)], LIGHT);
  picture.words(116, &[(20, 90), (120, 60)], LIGHT);
  picture.words(152, &[(20, 70), (100, 60)], LIGHT);
  let selection = picture.select(point(40.0, 84.0), point(380.0, 162.0));
  let first = selection.bands[0];
  assert!(first.right >= 350.0, "{:?}", selection.bands);
}
