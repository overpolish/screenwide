// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A highlight's grips, as the native chrome needs them.

#[cfg(target_os = "windows")]
use super::geometry::HighlightFlow;
use super::model::HighlightBand;
use crate::editor::annotations::handles::NativeAnnotationHandles;
use crate::editor::annotations::{AnnotationKind, AnnotationPoint};

/// A highlight reads the arrow's slots as its flow of bands, all normalised
/// over the source: `start` is the first band's top-left corner and `end` the
/// last band's bottom-right; `middle_x` and `middle_y` are how far left and
/// right the block of bands reaches, both across; `start_head` is the first
/// band's bottom and `end_head` the last band's top, both down. The native
/// side places its two grips - the selection's two ends - from those, and
/// picks it through [`flow`].
pub(crate) fn grips(
  start: AnnotationPoint,
  end: AnnotationPoint,
  bands: &[HighlightBand],
  index: u32,
  source: (u32, u32),
) -> NativeAnnotationHandles {
  let across = f64::from(source.0.max(1));
  let down = f64::from(source.1.max(1));
  let fallback = HighlightBand::between(start, end);
  let first = bands.first().copied().unwrap_or(fallback);
  let last = bands.last().copied().unwrap_or(fallback);
  let left = bands
    .iter()
    .map(|band| band.left)
    .fold(first.left, f64::min);
  let right = bands
    .iter()
    .map(|band| band.right)
    .fold(first.right, f64::max);
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Highlight.raw(),
    flags: 0,
    start_x: first.left / across,
    start_y: first.top / down,
    middle_x: left / across,
    middle_y: right / across,
    end_x: last.right / across,
    end_y: last.bottom / down,
    start_head: first.bottom / down,
    end_head: last.top / down,
    width: 0.0,
    radius: 0.0,
  }
}

/// The flow a grips' record carries, placed in the pixels `place` carries a
/// normalised point into.
#[cfg(target_os = "windows")]
pub(crate) fn flow(
  record: &NativeAnnotationHandles,
  place: impl Fn(f64, f64) -> [f32; 2],
) -> HighlightFlow {
  let start = place(record.start_x, record.start_y);
  let end = place(record.end_x, record.end_y);
  let block_left = place(record.middle_x, record.start_y);
  let block_right = place(record.middle_y, record.start_y);
  let first_bottom = place(record.start_x, record.start_head);
  let last_top = place(record.end_x, record.end_head);
  HighlightFlow {
    start,
    first_bottom: first_bottom[1],
    last_top: last_top[1],
    end,
    block: [block_left[0], block_right[0]],
  }
}
