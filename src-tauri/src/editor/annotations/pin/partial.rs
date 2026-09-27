// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A path shown while it is still being worked out.
//!
//! The legs nearest the playhead are followed first, and each one that lands
//! is shown straight away over the path the annotation had, which stands in
//! for the legs still to come. The patchwork is only ever shown; it is never
//! kept as the pin's path.

use super::leg::NO_COVER;
use super::resolve::{PinSample, PinnedPath};

/// `partial` wherever it has frames, and `stale` everywhere else, across a
/// clip from `start_ms` to `end_ms`. `key` must differ from every other
/// path's, the finished one's included, since what is read along a path is
/// kept by its key.
pub(crate) fn patched(
  partial: &PinnedPath,
  stale: &PinnedPath,
  start_ms: u64,
  end_ms: u64,
  key: u64,
) -> PinnedPath {
  let grown = !partial.growth.is_empty() || !stale.growth.is_empty();
  let viewed = !partial.views.is_empty() || !stale.views.is_empty();
  // Each moment's growth and view come with it from the path it is taken
  // from.
  let taken = |path: &PinnedPath, index: usize| {
    (
      path.samples[index],
      path.growth.get(index).copied().unwrap_or([0.0; 4]),
      path.views.get(index).copied().unwrap_or(NO_COVER),
    )
  };
  let mut merged: Vec<(PinSample, [f32; 4], [f32; 4])> =
    Vec::with_capacity(stale.samples.len().max(partial.samples.len()));
  let (mut fresh, mut old) = (0, 0);
  // Both are in time order: take each moment from the fresh path where it
  // has one.
  while fresh < partial.samples.len() || old < stale.samples.len() {
    let next_fresh = partial.samples.get(fresh);
    let next_old = stale.samples.get(old);
    match (next_fresh, next_old) {
      (Some(a), Some(b)) if a.ms <= b.ms => {
        if a.ms == b.ms {
          old += 1;
        }
        merged.push(taken(partial, fresh));
        fresh += 1;
      }
      (_, Some(_)) => {
        merged.push(taken(stale, old));
        old += 1;
      }
      (Some(_), None) => {
        merged.push(taken(partial, fresh));
        fresh += 1;
      }
      (None, None) => break,
    }
  }
  let mut shown = Vec::new();
  let mut from: Option<usize> = None;
  for index in 0..=merged.len() {
    let visible = merged.get(index).is_some_and(|(sample, ..)| sample.visible);
    match (from, visible) {
      (None, true) => from = Some(index),
      (Some(first), false) => {
        let start = if first == 0 {
          start_ms
        } else {
          merged[first].0.ms
        };
        let end = merged.get(index).map_or(end_ms, |(sample, ..)| sample.ms);
        shown.push([start, end]);
        from = None;
      }
      _ => {}
    }
  }
  PinnedPath {
    key,
    growth: if grown {
      merged.iter().map(|(_, growth, _)| *growth).collect()
    } else {
      Vec::new()
    },
    views: if viewed {
      merged.iter().map(|(.., view)| *view).collect()
    } else {
      Vec::new()
    },
    samples: merged.into_iter().map(|(sample, ..)| sample).collect(),
    shown,
    weak: Vec::new(),
    hidden: Vec::new(),
    covered: Vec::new(),
    under: Vec::new(),
  }
}
