// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A redaction's retained draw record.
//!
//! `p0` is the box's top-left corner and `p1` and `p2` its bottom-right, in
//! source pixels; the compositor snaps them outward to whole pixels. `color`
//! is the opaque fill, `flags` carries `PIXELATE`, `MOSAIC` or `BLUR`, and
//! `width` the corner radius as a percentage of the box's shorter side,
//! which the compositor and the halo each turn into pixels of their own.
//! `params` holds the block or cell size in source pixels and the seed's
//! high and low halves, each exact in a float. `p3` is the grid a pixelated
//! or blurred box draws from: a secure pixelation's zones across and blocks
//! per zone, or a classic pixelation's or a blur's cells across and down.
//! Its entries, two floats each - a zone's two [`super::palette::packed`]
//! inks, or a cell's packed colour and a `-1` - are `data_count` points from
//! `data_offset` in the side buffer's points.

use super::held::HeldFill;
use crate::editor::annotations::flags::{BLUR, MOSAIC, PIXELATE};
use crate::editor::annotations::{
  annotation_colour, AnnotationPoint, AnnotationRedaction, AnnotationStyle,
};
use std::borrow::Cow;

/// Where a list of redactions reads what is under and around each box.
#[derive(Clone, Copy)]
pub(crate) enum RedactSource<'a> {
  /// Nothing to read: erase and pixelate fall back to the style's colour.
  None,
  /// A screenshot's own pixels, read for every fill.
  Picture(&'a RedactPicture<'a>),
  /// A recording's frame. Each fill is the one held from its clip's first
  /// frame, and each box snaps outward to even pixels, since a video keeps
  /// its colour at half resolution and an odd edge would leave a colour
  /// sample half covered.
  Video { source_per_output: f64 },
}

/// The picture a list of redactions covers: the pixels erase and pixelate
/// take their colour from, and how many source pixels one output pixel is,
/// which sizes the blocks.
#[derive(Clone, Copy)]
pub(crate) struct RedactPicture<'a> {
  pub(crate) rgba: &'a [u8],
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) source_per_output: f64,
}

impl<'a> RedactPicture<'a> {
  /// `rgba` is `width` by `height` source pixels, drawn `image_width` output
  /// pixels wide on its canvas.
  pub(crate) fn new(rgba: &'a [u8], width: u32, height: u32, image_width: f64) -> Self {
    Self {
      rgba,
      width,
      height,
      source_per_output: crate::editor::annotations::snap::source_per_output(
        (width, height),
        image_width,
      ),
    }
  }
}

/// How a redaction's record is filled, and the grid a pixelated or blurred
/// one draws from, which travels in the side buffer beside the record.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RedactFill {
  pub(crate) color: [f32; 4],
  pub(crate) flags: u32,
  pub(crate) params: [f32; 3],
  /// The corner radius, a percentage of the box's shorter side.
  pub(crate) radius: f32,
  pub(crate) grid: [u32; 2],
  pub(crate) entries: Vec<[f32; 2]>,
}

pub(crate) fn draw_points(start: AnnotationPoint, end: AnnotationPoint) -> [[f32; 2]; 3] {
  let min = [start.x.min(end.x) as f32, start.y.min(end.y) as f32];
  let max = [start.x.max(end.x) as f32, start.y.max(end.y) as f32];
  [min, max, max]
}

/// The whole source pixels the compositor paints for the box, `[x0, y0, x1,
/// y1)`, read from the same single-precision corners the record carries so
/// the ring sampled here always lies outside what the kernel covers. The
/// twin of `screenwide_redactions` in `gpu_compositor_macos_redact.m`.
pub(crate) fn painted_bounds(corners: [[f32; 2]; 3], width: u32, height: u32) -> [u32; 4] {
  let clamp = |value: f32, limit: u32| value.max(0.0).min(limit as f32) as u32;
  [
    clamp(corners[0][0].floor(), width),
    clamp(corners[0][1].floor(), height),
    clamp(corners[2][0].ceil(), width),
    clamp(corners[2][1].ceil(), height),
  ]
}

/// What a redaction reads of the picture: the surface around the box, and a
/// secure pixelation's zones. A screenshot reads it for every fill, a
/// recording once from its clip's first frame.
pub(crate) fn held_fill(
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
  picture: &RedactPicture<'_>,
) -> HeldFill {
  let bounds = painted_bounds(draw_points(start, end), picture.width, picture.height);
  let surface = (style.redaction != AnnotationRedaction::Color)
    .then(|| {
      crate::editor::surface_colour::surrounding_colour(
        picture.rgba,
        picture.width,
        picture.height,
        bounds,
      )
    })
    .flatten();
  if style.redaction != AnnotationRedaction::Pixelate {
    return HeldFill {
      surface,
      ..HeldFill::default()
    };
  }
  let block = block_size(style, picture.source_per_output);
  let own = own_bytes(style);
  let zones = super::palette::zones(
    picture.rgba,
    picture.width,
    bounds,
    surface.unwrap_or(own),
    block,
  );
  HeldFill {
    surface,
    columns: zones.columns,
    blocks: zones.blocks,
    inks: zones.inks,
    surfaces: Vec::new(),
  }
}

/// The record's fill. Colour mode fills with the style's colour; the other
/// modes with the surface around the box, or the style's colour where there
/// is nothing to read or nothing opaque around the box. Every fill is
/// opaque: an alpha taken from the covered pixels would carry their shape. A
/// secure pixelation also takes, zone by zone, the few colours each zone
/// covers; a classic pixelation or a blur only the size of its cells, which
/// the compositor averages. `held` is a recording's fill from its clip's
/// first frame.
pub(crate) fn fill(
  start: AnnotationPoint,
  end: AnnotationPoint,
  seed: u32,
  style: &AnnotationStyle,
  source: RedactSource<'_>,
  held: Option<&HeldFill>,
) -> RedactFill {
  let (read, per_output) = match source {
    RedactSource::Picture(picture) => (
      Some(Cow::Owned(held_fill(start, end, style, picture))),
      picture.source_per_output,
    ),
    RedactSource::Video { source_per_output } => (held.map(Cow::Borrowed), source_per_output),
    RedactSource::None => (None, 1.0),
  };
  let per_output = if per_output.is_finite() && per_output > 0.0 {
    per_output
  } else {
    1.0
  };
  let surrounding = read
    .as_ref()
    .and_then(|read| read.surface)
    .filter(|_| style.redaction != AnnotationRedaction::Color);
  let (flags, size, grid, entries) = match style.redaction {
    AnnotationRedaction::Pixelate => {
      let (grid, inks) = read.map_or(([0, 0], Vec::new()), |read| {
        let read = read.into_owned();
        ([read.columns, read.blocks], read.inks)
      });
      (PIXELATE, block_size(style, per_output), grid, inks)
    }
    // Classic pixelation and blur are averaged by the GPU from the pixels
    // themselves, so all they carry is the size of their cells.
    AnnotationRedaction::PixelateClassic => (
      MOSAIC,
      super::cells::mosaic_cell(style.width, per_output) as f32,
      [0, 0],
      Vec::new(),
    ),
    AnnotationRedaction::Blur => (
      BLUR,
      super::cells::blur_cell(style.strength, per_output) as f32,
      [0, 0],
      Vec::new(),
    ),
    AnnotationRedaction::Erase | AnnotationRedaction::Color => (0, 0.0, [0, 0], Vec::new()),
  };
  let [red, green, blue] = surrounding
    .unwrap_or_else(|| own_bytes(style))
    .map(|channel| f32::from(channel) / 255.0);
  RedactFill {
    color: [red, green, blue, 1.0],
    flags,
    params: [size, f32::from((seed >> 16) as u16), f32::from(seed as u16)],
    radius: if style.radius.is_finite() {
      style.radius.clamp(0.0, 50.0) as f32
    } else {
      0.0
    },
    grid,
    entries,
  }
}

/// A secure pixelation's block, in source pixels.
fn block_size(style: &AnnotationStyle, source_per_output: f64) -> f32 {
  (style.width * source_per_output).max(1.0) as f32
}

/// The style's own colour as opaque bytes: the fill wherever there is no
/// surface to take.
fn own_bytes(style: &AnnotationStyle) -> [u8; 3] {
  let own = annotation_colour(&style.color);
  [own[0], own[1], own[2]].map(|channel| (channel * 255.0).round() as u8)
}
