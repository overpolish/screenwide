// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The colours a pixelated redaction draws its blocks in.
//!
//! The box is cut into zones at least as tall as the box and never less than
//! two blocks square. A box drawn round a line of type is about a line high,
//! and a line is taller than a character is wide at any size, so a zone
//! always spans more than one character however large the type is against
//! the blocks. Each zone reads the covered pixels for one thing: which few
//! colours it holds, such as the ink of the word under it. Not where inside
//! the zone any colour sits, and not how much of it there is beyond the cut
//! that makes it one of the few: the colours are handed on in an order of
//! their own, and which block takes which is the seed's choice. So the
//! colours sit roughly where they did - green over one word, white over the
//! next - while no block carries a glyph's shape, which leaves a
//! depixelation attack, matching block patterns against rendered candidates,
//! nothing finer than the zones to match. What does show is which colour
//! sits where to the nearest zone, and which zones are empty: roughly where
//! the words break. A box over several lines has zones as tall as all of
//! them, so its colours follow the words across the box but not down it.
//! The attack tests in `cursor_export/platform_macos_redact_attack_tests.rs`
//! measure how much shows.

/// How many colours besides the surface a zone draws with.
pub(crate) const INKS: usize = 2;

/// The fewest blocks a zone's side spans, whatever the box's height.
const ZONE_BLOCKS: u32 = 2;
/// The most zones one box is cut into; a larger box takes larger zones.
const MAX_ZONES: u64 = 4_096;
/// At most this many pixels are read across the whole box, so a drag that
/// repacks the box every sample stays cheap however large it is.
const MAX_SAMPLES: u64 = 262_144;
/// The fewest a zone reads, however many zones share the budget.
const MIN_ZONE_SAMPLES: u64 = 256;
/// Colours are grouped by their top four bits a channel, so the antialiased
/// edge of a glyph votes with its core rather than scattering across shades.
const LEVELS: usize = 16;
/// The smallest share of a zone a colour needs to count as one of its own.
const MIN_SHARE: f64 = 0.02;
/// How far apart, in any one channel, two colours must be to read as two.
const APART: i32 = 40;

/// A box's zones: `columns` across, each `blocks` blocks square, and every
/// zone's inks in rows from the top-left, as [`packed`] carries them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Zones {
  pub(crate) columns: u32,
  pub(crate) blocks: u32,
  pub(crate) inks: Vec<[f32; 2]>,
}

/// The zones of the box `[x0, y0, x1, y1)`, pixelated with blocks `block`
/// source pixels square and counted from the box's top-left, against
/// `surface`. The zone edges fall on block edges exactly as the kernel's
/// block index divides them.
pub(crate) fn zones(
  rgba: &[u8],
  width: u32,
  bounds: [u32; 4],
  surface: [u8; 3],
  block: f32,
) -> Zones {
  let [x0, y0, x1, y1] = bounds;
  let (across, down) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
  if across == 0 || down == 0 || !block.is_finite() || block <= 0.0 {
    return Zones::default();
  }
  let count = |blocks: u32| {
    let size = block * blocks as f32;
    let count = |extent: u32| (extent as f32 / size).ceil().max(1.0) as u32;
    (count(across), count(down))
  };
  let mut blocks = ZONE_BLOCKS.max((down as f32 / block).ceil() as u32);
  while {
    let (columns, rows) = count(blocks);
    u64::from(columns) * u64::from(rows) > MAX_ZONES
  } {
    blocks *= 2;
  }
  let (columns, rows) = count(blocks);
  let size = block * blocks as f32;
  let edge =
    |index: u32, origin: u32, limit: u32| (origin + (index as f32 * size).ceil() as u32).min(limit);
  let mut palette = Palette::default();
  let budget = (MAX_SAMPLES / (u64::from(columns) * u64::from(rows))).max(MIN_ZONE_SAMPLES);
  let mut inks = Vec::with_capacity((columns * rows) as usize);
  for row in 0..rows {
    for column in 0..columns {
      let zone = [
        edge(column, x0, x1),
        edge(row, y0, y1),
        edge(column + 1, x0, x1),
        edge(row + 1, y0, y1),
      ];
      let found = palette.inks(rgba, width, zone, surface, budget);
      inks.push([packed(found.first()), packed(found.get(1))]);
    }
  }
  Zones {
    columns,
    blocks,
    inks,
  }
}

/// One bucket table, cleared between zones rather than made again for each.
pub(crate) struct Palette {
  buckets: Vec<(u32, [u64; 3])>,
  touched: Vec<usize>,
}

impl Default for Palette {
  fn default() -> Self {
    Self {
      buckets: vec![(0, [0; 3]); LEVELS * LEVELS * LEVELS],
      touched: Vec::new(),
    }
  }
}

impl Palette {
  /// The colours under `[x0, y0, x1, y1)` that stand apart from `surface`,
  /// read from at most about `samples` pixels, the most common first choice
  /// and at most [`INKS`] of them, handed back darkest first.
  pub(crate) fn inks(
    &mut self,
    rgba: &[u8],
    width: u32,
    bounds: [u32; 4],
    surface: [u8; 3],
    samples: u64,
  ) -> Vec<[u8; 3]> {
    let [x0, y0, x1, y1] = bounds;
    let area = u64::from(x1.saturating_sub(x0)) * u64::from(y1.saturating_sub(y0));
    if area == 0 {
      return Vec::new();
    }
    for index in self.touched.drain(..) {
      self.buckets[index] = (0, [0; 3]);
    }
    let step = ((area as f64 / samples.max(1) as f64).sqrt().ceil() as usize).max(1);
    let mut total = 0_u32;
    for y in (y0..y1).step_by(step) {
      for x in (x0..x1).step_by(step) {
        let offset = ((y as usize * width as usize) + x as usize) * 4;
        let Some(&[red, green, blue, 255]) = rgba.get(offset..offset + 4) else {
          continue;
        };
        let index = usize::from(red >> 4) * LEVELS * LEVELS
          + usize::from(green >> 4) * LEVELS
          + usize::from(blue >> 4);
        let bucket = &mut self.buckets[index];
        if bucket.0 == 0 {
          self.touched.push(index);
        }
        bucket.0 += 1;
        for (sum, channel) in bucket.1.iter_mut().zip([red, green, blue]) {
          *sum += u64::from(channel);
        }
        total += 1;
      }
    }
    let floor = (f64::from(total) * MIN_SHARE).max(1.0);
    let mut candidates: Vec<(u32, [u8; 3])> = self
      .touched
      .iter()
      .map(|index| self.buckets[*index])
      .filter(|(count, _)| f64::from(*count) >= floor)
      .map(|(count, sums)| (count, sums.map(|sum| (sum / u64::from(count)) as u8)))
      .filter(|(_, colour)| !alike(*colour, surface))
      .collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut chosen: Vec<[u8; 3]> = Vec::with_capacity(INKS);
    for (_, colour) in candidates {
      if chosen.len() == INKS {
        break;
      }
      if chosen.iter().all(|kept| !alike(*kept, colour)) {
        chosen.push(colour);
      }
    }
    chosen.sort_by_key(|colour| luminance(*colour));
    chosen
  }
}

fn alike(a: [u8; 3], b: [u8; 3]) -> bool {
  a.iter()
    .zip(b)
    .all(|(a, b)| (i32::from(*a) - i32::from(b)).abs() < APART)
}

fn luminance(colour: [u8; 3]) -> u32 {
  2126 * u32::from(colour[0]) + 7152 * u32::from(colour[1]) + 722 * u32::from(colour[2])
}

/// An ink as one float the record carries exactly: 24 bits of RGB, which a
/// float holds without rounding. The twin of `redaction_ink` in the
/// `redact_source_rgba` kernel; `-1` is no ink.
pub(crate) fn packed(ink: Option<&[u8; 3]>) -> f32 {
  ink.map_or(-1.0, |[red, green, blue]| {
    ((u32::from(*red) << 16) | (u32::from(*green) << 8) | u32::from(*blue)) as f32
  })
}
