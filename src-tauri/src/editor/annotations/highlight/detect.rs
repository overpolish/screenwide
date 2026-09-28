// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding the lines of text under a selection, and the page they sit on.
//!
//! The picture is read the way a scan is segmented: the page's surface is the
//! colour most of the neighbourhood is, anything far enough from it is ink,
//! and the rows that carry ink are the lines. Along a line, ink columns closer
//! than a word space are one word and closer than a column gap are one run of
//! text, which is how a selection's ends settle on a word's edge when let go
//! near one and its middle lines reach their own ends. Nothing here knows what
//! the text says, so it reads any script, code and interface labels alike.
//!
//! Every length is a share of the lines' own height, so the same selection
//! reads a 1x capture and a 2x one the same way.

use super::model::{HighlightBand, HighlightTone};
use crate::editor::annotations::AnnotationPoint;
use page::Page;

mod ink;
mod lines;
mod page;

/// RGBA pixels, `width` by `height`, straight alpha.
#[derive(Clone, Copy)]
pub(crate) struct HighlightPixels<'a> {
  pub(crate) rgba: &'a [u8],
  pub(crate) width: u32,
  pub(crate) height: u32,
}

/// What a selection covers: its bands in reading order, and the page they
/// were read from.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Selection {
  pub(crate) bands: Vec<HighlightBand>,
  pub(crate) tone: HighlightTone,
}

/// How far a channel may stray from the surface before the pixel is ink, out
/// of 255. Anti-aliased edges fall either side; the cores of glyphs are far
/// past it on any page a person could read.
const INK_DISTANCE: i32 = 56;
/// A gap inside a line narrower than this share of the line height joins two
/// glyphs into one word.
const WORD_GAP: f64 = 0.16;
/// A gap narrower than this share of the line height joins two words into one
/// run of text; a wider one is a column, a cell or the end of the line.
const RUN_GAP: f64 = 1.2;
/// An end let go inside a word within this share of the line height of the
/// word's edge settles on that edge; further in, it cuts the word there.
const EDGE_SNAP: f64 = 0.25;
/// How far a band reaches past its line's ink, as shares of the line height.
const PAD_X: f64 = 0.12;
const PAD_Y: f64 = 0.16;
/// The least a band's height may be as a share of the tallest line's, so a
/// line of short letters is not highlighted as a thinner stroke.
const EVEN_HEIGHT: f64 = 0.8;
/// How many pixels the surface is estimated from at most.
const SURFACE_SAMPLES: f64 = 60_000.0;
/// How far a line is looked along for its ends, in line heights.
const LINE_REACH: f64 = 60.0;
/// The least share of the neighbourhood of the press the page has to be. Less
/// is a photo or a gradient, with no page for text to sit on.
const PAGE_SHARE: f64 = 0.5;
/// How long a stretch of ink along one row may run, in band heights, before
/// it is a picture, a pane or a bar rather than part of a letter.
const SOLID_RUN: f64 = 2.0;
/// How tall a line may be, in band heights, before it is ink the page was
/// misread around rather than text.
const TALLEST_LINE: f64 = 3.0;

/// The selection pressed at `start` and let go at `end`, both in source
/// pixels. `pixels` is the picture it reads, `scale` its pixels per source
/// pixel, and `height` the band's height in source pixels wherever no text is
/// found. Without pixels there is nothing to read, and the selection is one
/// band along the drag.
pub(crate) fn select(
  pixels: Option<(HighlightPixels<'_>, (f64, f64))>,
  start: AnnotationPoint,
  end: AnnotationPoint,
  height: f64,
) -> Selection {
  let height = if height.is_finite() && height > 0.0 {
    height
  } else {
    1.0
  };
  let Some((pixels, scale)) = pixels.filter(|(pixels, scale)| usable(pixels, *scale)) else {
    return Selection {
      bands: vec![plain_band(start, end, height)],
      tone: HighlightTone::default(),
    };
  };
  let to_picture = |point: AnnotationPoint| (point.x * scale.0, point.y * scale.1);
  let to_source = |band: [f64; 4]| HighlightBand {
    left: band[0] / scale.0,
    top: band[1] / scale.1,
    right: band[2] / scale.0,
    bottom: band[3] / scale.1,
  };
  let page = Page::new(pixels);
  let from = to_picture(start);
  let to = to_picture(end);
  let reach = (height * scale.1).max(6.0);
  let Some(surface) = page.surface_at(from, reach) else {
    return Selection {
      bands: vec![plain_band(start, end, height)],
      tone: HighlightTone::UNREAD,
    };
  };
  let bands = page
    .read(from, to, reach, surface)
    .map(|bands| bands.into_iter().map(to_source).collect::<Vec<_>>());
  match bands {
    Some(bands) if !bands.is_empty() => {
      let tone = page.tone(surface, &bands, scale);
      Selection { bands, tone }
    }
    _ => {
      let band = plain_band(start, end, height);
      let tone = page.tone(surface, std::slice::from_ref(&band), scale);
      Selection {
        bands: vec![band],
        tone,
      }
    }
  }
}

fn usable(pixels: &HighlightPixels<'_>, scale: (f64, f64)) -> bool {
  let expected = (pixels.width as usize)
    .checked_mul(pixels.height as usize)
    .and_then(|count| count.checked_mul(4));
  expected.is_some_and(|expected| pixels.rgba.len() >= expected && expected > 0)
    && scale.0.is_finite()
    && scale.1.is_finite()
    && scale.0 > 0.0
    && scale.1 > 0.0
}

/// One band along the drag, `height` tall and centred on where it began.
fn plain_band(start: AnnotationPoint, end: AnnotationPoint, height: f64) -> HighlightBand {
  let half = height * 0.5;
  HighlightBand {
    left: start.x.min(end.x),
    top: start.y - half,
    right: start.x.max(end.x),
    bottom: start.y + half,
  }
}

/// The luminance the compositor weighs a colour by, over encoded channels.
fn luma(rgb: [f64; 3]) -> f64 {
  0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// The ink held at least a fifth of the scale from the page, so the
/// recolouring never divides by a difference that is not there. Ink that
/// barely stands out is pushed away from the page, towards the far end.
fn apart(page: f64, ink: f64) -> f64 {
  const LEAST: f64 = 0.2;
  if (ink - page).abs() >= LEAST {
    return ink;
  }
  if page > 0.5 {
    (page - LEAST).max(0.0)
  } else {
    (page + LEAST).min(1.0)
  }
}

#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;
