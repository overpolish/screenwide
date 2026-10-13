// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captions drawn by the canvas shader: a word's type in its own colour with
//! no box, the Box template's backdrop, the Highlight template's pill, and
//! type set in the face the style names.

use screenwide_annotations::text::face::TypeFace;
use screenwide_captions::{
  caption_pieces, Caption, CaptionFont, CaptionFrame, CaptionPiece, CaptionStyle, CaptionTemplate,
  CaptionWord, RecordingCaptions,
};
use screenwide_imaging::CapturedImage;

use super::render_test_helpers::{compositor, gpu, target, view, OUTPUT};
use super::*;
use crate::annotation_gpu::{caption_arrows, caption_face, caption_measure};
use screenwide_output::fixtures::test_output_settings;

/// A mid grey picture filling the canvas, which neither white type nor a
/// dark box disappears into.
fn grey() -> CapturedImage {
  CapturedImage {
    width: OUTPUT.0,
    height: OUTPUT.1,
    rgba: [128, 128, 128, 255].repeat((OUTPUT.0 * OUTPUT.1) as usize),
  }
}

fn word(text: &str, start_ms: f64) -> CaptionWord {
  CaptionWord {
    text: text.to_owned(),
    start_ms,
    end_ms: start_ms + 400.0,
    source_start_ms: start_ms,
    source_end_ms: start_ms + 400.0,
  }
}

fn caption() -> Caption {
  Caption {
    id: 0,
    start_ms: 0.0,
    end_ms: 2_000.0,
    text: "WWW MMM".to_owned(),
    lines: vec![vec![word("WWW", 0.0), word("MMM", 500.0)]],
    spaced: true,
  }
}

/// `template`'s style, with the word being said coloured.
fn styled(template: CaptionTemplate) -> CaptionStyle {
  CaptionStyle {
    highlight_words: true,
    ..template.style()
  }
}

/// The pieces `style` draws 700 ms in, while "MMM" is being said, and the
/// face they are set in.
fn pieces(style: CaptionStyle) -> (Vec<CaptionPiece>, TypeFace) {
  let face = caption_face(&style.font);
  let recording = RecordingCaptions {
    enabled: true,
    style,
    ..Default::default()
  };
  let pieces = caption_pieces(
    &[caption()],
    &recording,
    CaptionFrame {
      width: OUTPUT.0,
      height: OUTPUT.1,
    },
    700.0,
    &mut caption_measure(&face),
  );
  (pieces, face)
}

/// The canvas with `pieces` set in `face` drawn over the grey picture, which
/// fills it, BGRA.
fn drawn((pieces, face): &(Vec<CaptionPiece>, TypeFace)) -> Vec<u8> {
  let compositor = compositor();
  let image = grey();
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  let (width, height) = (f64::from(OUTPUT.0), f64::from(OUTPUT.1));
  (settings.crop_x, settings.crop_y) = (0.0, 0.0);
  (settings.crop_width, settings.crop_height) = (width, height);
  (settings.image_x, settings.image_y, settings.image_width) = (0.0, 0.0, width);
  let source = compositor.screenshot_source(&image).unwrap();
  let mut prepared = crate::annotation_gpu::prepared_arrows(
    &[],
    (image.width, image.height),
    &settings,
    Some(&image),
    None,
    None,
  )
  .unwrap();
  prepared.draw_above(caption_arrows(pieces, face));
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
    )
    .unwrap();
  gpu().read_texture(&output).unwrap()
}

/// The pixels within `half` of `centre`, RGB.
fn around(canvas: &[u8], centre: [f64; 2], half: [f64; 2]) -> Vec<[u8; 3]> {
  let (x0, x1) = ((centre[0] - half[0]) as u32, (centre[0] + half[0]) as u32);
  let (y0, y1) = ((centre[1] - half[1]) as u32, (centre[1] + half[1]) as u32);
  (y0..y1)
    .flat_map(|y| (x0..x1).map(move |x| (y * OUTPUT.0 + x) as usize * 4))
    .map(|at| [canvas[at + 2], canvas[at + 1], canvas[at]])
    .collect()
}

fn is_yellow([r, g, b]: [u8; 3]) -> bool {
  r > 220 && g > 180 && b < 80
}

fn is_white([r, g, b]: [u8; 3]) -> bool {
  r > 230 && g > 230 && b > 230
}

fn is_dark([r, g, b]: [u8; 3]) -> bool {
  r < 70 && g < 70 && b < 70
}

#[test]
fn draws_a_caption_word_in_its_own_colour_with_a_dark_outline_and_no_box() {
  let set = pieces(styled(CaptionTemplate::Clean));
  let canvas = drawn(&set);
  let [first, said] = [&set.0[0], &set.0[1]];
  let size = first.size;
  let first_pixels = around(&canvas, first.centre, [size, size * 0.4]);
  let said_pixels = around(&canvas, said.centre, [size, size * 0.4]);
  assert!(first_pixels.iter().any(|pixel| is_white(*pixel)));
  assert!(first_pixels.iter().any(|pixel| is_dark(*pixel)));
  assert!(said_pixels.iter().any(|pixel| is_yellow(*pixel)));
  // No box: just outside the type, the picture shows as it does further
  // along the same row.
  let y = first.centre[1] - size * 0.75;
  let beside = around(&canvas, [first.centre[0], y], [2.0, 2.0]);
  let further = around(&canvas, [first.centre[0] - size * 3.0, y], [2.0, 2.0]);
  assert_eq!(beside, further);
}

#[test]
fn draws_the_box_template_on_a_dark_backdrop_and_the_highlight_on_a_pill() {
  let boxed = pieces(styled(CaptionTemplate::Box));
  let canvas = drawn(&boxed);
  // Between the two words, where there is no type, the backdrop shows.
  let between = [
    (boxed.0[1].centre[0] + boxed.0[2].centre[0]) / 2.0,
    boxed.0[1].centre[1],
  ];
  assert!(around(&canvas, between, [1.0, 1.0])
    .iter()
    .all(|pixel| is_dark(*pixel)));

  let pill = pieces(styled(CaptionTemplate::Highlight));
  let canvas = drawn(&pill);
  let said = pill.0.last().unwrap();
  let on_pill = around(&canvas, said.centre, [said.size, said.size * 0.4]);
  assert!(on_pill.iter().any(|pixel| is_yellow(*pixel)));
  assert!(on_pill.iter().any(|pixel| is_dark(*pixel)));
}

/// How many white pixels the caption's words leave on the canvas.
fn white_count(set: &(Vec<CaptionPiece>, TypeFace)) -> usize {
  let canvas = drawn(set);
  set
    .0
    .iter()
    .flat_map(|piece| around(&canvas, piece.centre, [piece.size * 2.0, piece.size * 0.6]))
    .filter(|pixel| is_white(*pixel))
    .count()
}

#[test]
fn sets_the_type_in_the_family_and_weight_the_style_names() {
  let in_font = |family: Option<&str>, weight| {
    pieces(CaptionStyle {
      font: CaptionFont {
        family: family.map(str::to_owned),
        weight,
      },
      highlight_words: false,
      ..CaptionTemplate::Clean.style()
    })
  };
  // Georgia ships with both macOS and Windows, and its words are wider than
  // Inter's, so they are laid out further apart.
  let inter = in_font(None, 600);
  let georgia = in_font(Some("Georgia"), 400);
  let spread = |set: &(Vec<CaptionPiece>, TypeFace)| set.0[1].centre[0] - set.0[0].centre[0];
  assert!((spread(&inter) - spread(&georgia)).abs() > 1.0);
  assert_ne!(drawn(&inter), drawn(&georgia));
  // A heavier weight lays down more ink.
  assert!(white_count(&in_font(None, 900)) > white_count(&in_font(None, 300)));
}

#[test]
fn fits_the_pill_to_type_set_in_another_face() {
  // Courier New ships with both macOS and Windows and sets "MMM" at two
  // thirds of Inter's width: a pill sized by Inter would reach far past it.
  let set = pieces(CaptionStyle {
    font: CaptionFont {
      family: Some("Courier New".to_owned()),
      weight: 700,
    },
    ..CaptionTemplate::Highlight.style()
  });
  let canvas = drawn(&set);
  let said = set.0.last().unwrap();
  let neighbour = &set.0[1];
  // From halfway to the word before; its outline is told apart from the
  // said word's type by being only mostly opaque.
  let from = ((neighbour.centre[0] + said.centre[0]) / 2.0) as u32;
  let band = (said.centre[1] - said.size * 0.3) as u32..(said.centre[1] + said.size * 0.3) as u32;
  let columns = |test: fn([u8; 3]) -> bool| {
    let hits: Vec<u32> = (from..OUTPUT.0)
      .filter(|&x| {
        band.clone().any(|y| {
          let at = (y * OUTPUT.0 + x) as usize * 4;
          test([canvas[at + 2], canvas[at + 1], canvas[at]])
        })
      })
      .collect();
    (*hits.first().unwrap(), *hits.last().unwrap())
  };
  let is_black = |[r, g, b]: [u8; 3]| r < 8 && g < 8 && b < 8;
  let (pill, ink) = (columns(is_yellow), columns(is_black));
  // The pill reaches a little past its word on both sides: enough to hold
  // it, not so far it runs under the next.
  let reach = [ink.0 - pill.0, pill.1 - ink.1].map(|reach| f64::from(reach) / said.size);
  assert!(
    reach.iter().all(|reach| (0.1..0.3).contains(reach)),
    "pill {pill:?} ink {ink:?}"
  );
}
