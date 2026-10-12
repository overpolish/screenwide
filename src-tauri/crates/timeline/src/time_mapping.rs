// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::TimelineRange;

pub fn source_to_output_us(ranges: &[TimelineRange], source_us: u64) -> Option<u64> {
  let range = ranges.iter().find(|range| {
    source_us >= range.source_start_us
      && (source_us < range.source_end_us
        || source_us == range.source_end_us && range.source_end_us == range.source_start_us)
  })?;
  Some(range.output_start_us.saturating_add(
    ((source_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate).round() as u64,
  ))
}

/// Where `source_us` lands on the output timeline. A moment the timeline cut
/// away lands where its source predecessor ends: the end of the range that
/// finishes latest at or before it, which is where restoring it would put it
/// back. With no such range it lands at the start of the range earliest in
/// the source. Within one run of ranges in source order a later moment never
/// lands earlier; across a reordered timeline it may. No ranges means an
/// uncut timeline.
pub fn output_at_us(ranges: &[TimelineRange], source_us: f64) -> f64 {
  output_in(ranges, source_us, |start, end| {
    start <= source_us && source_us < end
  })
}

/// [`output_at_us`] for the end of a stretch: a moment that ends one range
/// and starts another lands at the end of the first, wherever the second
/// plays. On a timeline in source order the two are the same place.
pub fn output_end_at_us(ranges: &[TimelineRange], source_us: f64) -> f64 {
  output_in(ranges, source_us, |start, end| {
    start < source_us && source_us <= end
  })
}

/// Where `source_us` lands: in the range `holds` says has it, or else where
/// its source predecessor ends, or else at the start of the range earliest
/// in the source.
fn output_in(ranges: &[TimelineRange], source_us: f64, holds: impl Fn(f64, f64) -> bool) -> f64 {
  if ranges.is_empty() {
    return source_us;
  }
  let mut predecessor: Option<&TimelineRange> = None;
  let mut earliest = &ranges[0];
  for range in ranges {
    let start = range.source_start_us as f64;
    if holds(start, range.source_end_us as f64) {
      return range.output_start_us as f64 + (source_us - start) / range.playback_rate;
    }
    if range.source_end_us as f64 <= source_us
      && predecessor.is_none_or(|found| found.source_end_us < range.source_end_us)
    {
      predecessor = Some(range);
    }
    if range.source_start_us < earliest.source_start_us {
      earliest = range;
    }
  }
  predecessor.map_or(earliest.output_start_us as f64, |range| {
    range.output_start_us as f64
      + (range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate
  })
}

/// Whether playback carries on from `from_us` to the later source moment
/// `to_us` without a jump: every range holding source between them sits, in
/// source order, in one unbroken stretch of output. False when either moment
/// is cut away, or when a reordered timeline plays the two apart.
pub fn plays_through(ranges: &[TimelineRange], from_us: u64, to_us: u64) -> bool {
  let holds =
    |range: &TimelineRange, us: u64| range.source_start_us <= us && us < range.source_end_us;
  let (Some(first), Some(last)) = (
    ranges.iter().position(|range| holds(range, from_us)),
    ranges.iter().position(|range| holds(range, to_us)),
  ) else {
    return false;
  };
  first <= last
    && ranges[first..=last]
      .windows(2)
      .all(|pair| pair[0].source_end_us <= pair[1].source_start_us)
    && ranges[..first]
      .iter()
      .chain(&ranges[last + 1..])
      .all(|range| range.source_end_us <= from_us || to_us < range.source_start_us)
}

/// The stretches of source, `[start_us, end_us)`, a reader can play the
/// ranges through moving only forward: a new one begins wherever a
/// reordered timeline goes back to an earlier stretch of the source. A
/// timeline in source order is one.
pub fn ascending_source_pieces(ranges: &[TimelineRange]) -> Vec<[u64; 2]> {
  let mut pieces: Vec<[u64; 2]> = Vec::new();
  for range in ranges {
    match pieces.last_mut() {
      Some(piece) if range.source_start_us >= piece[1] => piece[1] = range.source_end_us,
      _ => pieces.push([range.source_start_us, range.source_end_us]),
    }
  }
  pieces
}

/// How fast the timeline plays `source_us`: its range's rate, or 1 where it
/// was cut away or nothing was cut.
pub fn rate_at(ranges: &[TimelineRange], source_us: f64) -> f64 {
  ranges
    .iter()
    .find(|range| {
      source_us >= range.source_start_us as f64 && source_us < range.source_end_us as f64
    })
    .map_or(1.0, |range| range.playback_rate)
}

pub fn output_to_source_us(ranges: &[TimelineRange], output_us: u64) -> Option<u64> {
  for range in ranges {
    let output_end_us = range.output_start_us.saturating_add(
      ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
        .round() as u64,
    );
    if output_us <= output_end_us {
      return Some(range.source_start_us.saturating_add(
        ((output_us.saturating_sub(range.output_start_us)) as f64 * range.playback_rate).round()
          as u64,
      ));
    }
  }
  ranges.last().map(|range| range.source_end_us)
}

/// [`output_to_source_us`] for the start of a stretch: where one range ends
/// and the next begins, the next one's first moment, since on a reordered
/// timeline the range ending there may sit anywhere in the source.
pub fn output_start_to_source_us(ranges: &[TimelineRange], output_us: u64) -> Option<u64> {
  for range in ranges {
    let output_end_us = range.output_start_us.saturating_add(
      ((range.source_end_us.saturating_sub(range.source_start_us)) as f64 / range.playback_rate)
        .round() as u64,
    );
    if output_us < output_end_us {
      return Some(range.source_start_us.saturating_add(
        ((output_us.saturating_sub(range.output_start_us)) as f64 * range.playback_rate).round()
          as u64,
      ));
    }
  }
  ranges.last().map(|range| range.source_end_us)
}

/// How long the timeline `ranges` lay out plays for; `None` with none.
pub fn output_duration_us(ranges: &[TimelineRange]) -> Option<u64> {
  ranges.last().map(|last| {
    last.output_start_us.saturating_add(
      ((last.source_end_us.saturating_sub(last.source_start_us)) as f64 / last.playback_rate)
        .round() as u64,
    )
  })
}

/// Converts a source-time animation anchor plus an output-time duration back
/// into a source coordinate. Timed lanes can map that coordinate normally and
/// still match animations whose duration must not stretch with playback rate.
pub fn source_after_output_duration_us(
  ranges: Option<&[TimelineRange]>,
  anchor_us: u64,
  duration_us: u64,
) -> Option<u64> {
  let Some(ranges) = ranges else {
    return Some(anchor_us.saturating_add(duration_us));
  };
  let output_anchor_us = source_to_output_us(ranges, anchor_us)?;
  output_to_source_us(ranges, output_anchor_us.saturating_add(duration_us))
}

/// The reverse of [`source_after_output_duration_us`]: the source coordinate
/// that lies an output-time duration BEFORE the anchor, so a fixed-length
/// animation can be scheduled to finish exactly at the anchor at any rate.
pub fn source_before_output_duration_us(
  ranges: Option<&[TimelineRange]>,
  anchor_us: u64,
  duration_us: u64,
) -> Option<u64> {
  let Some(ranges) = ranges else {
    return Some(anchor_us.saturating_sub(duration_us));
  };
  let output_anchor_us = source_to_output_us(ranges, anchor_us)?;
  output_to_source_us(ranges, output_anchor_us.saturating_sub(duration_us))
}

#[cfg(test)]
mod tests;
