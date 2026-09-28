// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a pinned annotation is drawn at a moment.

use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

use super::model::AnnotationPin;
use super::target::anchor_of;

/// Where the pinned content is at one frame, relative to where the
/// annotation was drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PinSample {
  pub(crate) ms: u64,
  /// How far the anchor has moved, in source pixels.
  pub(crate) dx: f32,
  pub(crate) dy: f32,
  /// How much larger the content has grown.
  pub(crate) scale: f32,
  /// How far the frame's match is to be trusted, from 0 to 1. Zero where the
  /// content was not found and its place is bridged from either side.
  pub(crate) confidence: f32,
  /// Whether the annotation shows at this frame.
  pub(crate) visible: bool,
}

/// A pin worked out across its clip.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PinnedPath {
  /// Which keyframes, clip and target the path was worked out for, so a path
  /// kept while its replacement is tracked is known for what it is.
  pub(crate) key: u64,
  pub(crate) samples: Vec<PinSample>,
  /// How far a redaction's box is grown on each side at each sample, left,
  /// top, right and bottom, in source pixels: out to everywhere its content
  /// may have been while it was lost. Empty for every other kind.
  pub(crate) growth: Vec<[f32; 4]>,
  /// The part of the frame no still cover lies over at each sample, left,
  /// top, right and bottom, in source pixels, infinite on each side without
  /// one: what a redaction's box is trimmed to, so it never hides the cover
  /// its content went under. Empty for every other kind.
  pub(crate) views: Vec<[f32; 4]>,
  /// The stretches the annotation shows for, `[start, end)` in source
  /// milliseconds.
  pub(crate) shown: Vec<[u64; 2]>,
  /// The stretches it shows for but its content was lost in, bridged from
  /// either side.
  pub(crate) weak: Vec<[u64; 2]>,
  /// The stretches its content was off the frame.
  pub(crate) hidden: Vec<[u64; 2]>,
  /// The stretches an arrow, counter or text box is hidden for because its
  /// content was out of sight on screen: under something, or lost; and
  /// those a redaction is hidden for because its content went under a cover.
  pub(crate) covered: Vec<[u64; 2]>,
  /// The stretches it is hidden for because its content went under a still
  /// cover.
  pub(crate) under: Vec<[u64; 2]>,
}

impl PinnedPath {
  /// The sample of the frame showing at `ms`: the last one at or before it,
  /// with the two milliseconds a frame may show early.
  fn index(&self, ms: u64) -> Option<usize> {
    if self.samples.is_empty() {
      return None;
    }
    Some(
      self
        .samples
        .partition_point(|sample| sample.ms <= ms + 2)
        .saturating_sub(1),
    )
  }

  /// Where the annotation was last shown before its content went out of
  /// sight, when `ms` falls in a stretch it is hidden for because of that:
  /// the offset "Show Here" puts it back at, for the hand to set right. A
  /// stretch at the very start of the clip takes the first frame it shows.
  pub(crate) fn last_seen(&self, ms: u64) -> Option<[f64; 2]> {
    let [start, _] = *self
      .covered
      .iter()
      .find(|[start, end]| (*start..*end).contains(&ms))?;
    let seen = self
      .samples
      .iter()
      .rev()
      .find(|sample| sample.ms < start && sample.visible)
      .or_else(|| self.samples.iter().find(|sample| sample.visible))?;
    Some([f64::from(seen.dx), f64::from(seen.dy)])
  }
}

/// Where a pinned annotation is drawn at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
  /// How far it has moved from where it was drawn, in source pixels.
  pub(crate) offset: [f64; 2],
  /// How much a redaction's box has grown with its content.
  pub(crate) scale: f64,
  /// How far a redaction's edges sit outside its drawn box by the hand, from
  /// the keyframes that resized it: left, top, right, bottom.
  pub(crate) edges: [f64; 4],
  pub(crate) grow: [f64; 4],
  /// What a redaction's box is trimmed to, from its path: see
  /// [`PinnedPath::views`].
  pub(crate) view: [f64; 4],
  /// The stretch it is showing through, which its arrival and leaving play
  /// over; `None` while it is hidden.
  pub(crate) shown: Option<[u64; 2]>,
}

/// A view with no cover on any side.
pub(crate) const NO_COVER: [f64; 4] = [
  f64::NEG_INFINITY,
  f64::NEG_INFINITY,
  f64::INFINITY,
  f64::INFINITY,
];

/// Where `clip`'s pinned annotation is at `ms`.
///
/// On a keyframe it is exactly where the keyframe puts it, whatever the path,
/// so an annotation dragged there stays under the hand while its new path is
/// worked out. Elsewhere it follows the path it has, which may still be the
/// path of the keyframes before a change; with no path yet it waits at the
/// nearest keyframe. Where the hand said its content is out of view, it is
/// hidden, path or not.
pub(crate) fn placement(clip: &RecordingAnnotationClip, pin: &AnnotationPin, ms: u64) -> Placement {
  let whole = [clip.start_ms, clip.end_ms];
  let out_of_view = pin
    .keyframe_at(ms)
    .map_or_else(|| pin.out_of_view_at(ms), |keyframe| keyframe.out_of_view);
  if out_of_view {
    return Placement {
      offset: [0.0; 2],
      scale: 1.0,
      edges: [0.0; 4],
      grow: [0.0; 4],
      view: NO_COVER,
      shown: None,
    };
  }
  let path = pin.path.as_deref();
  let view = |path: &PinnedPath, index: usize| {
    path
      .views
      .get(index)
      .map_or(NO_COVER, |view| view.map(f64::from))
  };
  let shown = |path: &PinnedPath| {
    path
      .shown
      .iter()
      .map(|span| [span[0].max(clip.start_ms), span[1].min(clip.end_ms)])
      .find(|span| span[0] <= ms + 2 && ms < span[1])
      .filter(|span| room_to_arrive(clip, span[0]))
  };
  if let Some(keyframe) = pin.keyframe_at(ms) {
    return Placement {
      offset: [keyframe.dx, keyframe.dy],
      scale: 1.0,
      edges: pin.edges_at(ms),
      grow: [0.0; 4],
      // Trimmed on a keyframe too, or the box would stand over the cover
      // for the one frame a correction was made on.
      view: path
        .and_then(|path| Some(view(path, path.index(ms)?)))
        .unwrap_or(NO_COVER),
      shown: path.and_then(shown).or(Some(whole)),
    };
  }
  if let Some((path, index)) = path.and_then(|path| Some((path, path.index(ms)?))) {
    let sample = &path.samples[index];
    return Placement {
      offset: [f64::from(sample.dx), f64::from(sample.dy)],
      scale: f64::from(sample.scale),
      edges: pin.edges_at(ms),
      grow: path
        .growth
        .get(index)
        .map_or([0.0; 4], |grow| grow.map(f64::from)),
      view: view(path, index),
      shown: shown(path),
    };
  }
  let offset = pin
    .nearest(ms)
    .map_or([0.0; 2], |keyframe| [keyframe.dx, keyframe.dy]);
  Placement {
    offset,
    scale: 1.0,
    edges: pin.edges_at(ms),
    grow: [0.0; 4],
    view: NO_COVER,
    shown: Some(whole),
  }
}

/// Whether content coming back onto the frame at `from` leaves its clip room
/// for the annotation to arrive and leave again. The clip's own start always
/// does: that is the arrival the clip was made for.
fn room_to_arrive(clip: &RecordingAnnotationClip, from: u64) -> bool {
  if from <= clip.start_ms || !clip.annotation.animated {
    return true;
  }
  let span = clip.annotation.shape.kind().reveal_span_ms(clip.path_ms);
  clip.end_ms.saturating_sub(from) as f32 >= span
}

/// `annotation` where `placement` puts it. A redaction's box also grows and
/// shrinks with its content, takes the size its keyframes resized it to, is
/// grown over doubtful frames, and is trimmed clear of any still cover its
/// content goes under; the other kinds keep their size, since they are drawn
/// at the canvas's scale.
pub(crate) fn displaced(annotation: &Annotation, placement: &Placement) -> Annotation {
  let [ax, ay] = anchor_of(annotation);
  let [dx, dy] = placement.offset;
  let redaction = matches!(annotation.shape, AnnotationShape::Redact { .. });
  let scale = if redaction { placement.scale } else { 1.0 };
  let mut out = annotation.clone();
  out.shape = annotation.shape.mapped(|point| AnnotationPoint {
    x: ax + dx + scale * (point.x - ax),
    y: ay + dy + scale * (point.y - ay),
  });
  if let AnnotationShape::Redact { start, end, .. } = &mut out.shape {
    let [left, top, right, bottom]: [f64; 4] =
      std::array::from_fn(|side| placement.edges[side] + placement.grow[side]);
    let [x_min, y_min, x_max, y_max] = placement.view;
    let (x0, x1) = (start.x.min(end.x) - left, start.x.max(end.x) + right);
    let (y0, y1) = (start.y.min(end.y) - top, start.y.max(end.y) + bottom);
    // Wholly under a cover, it closes up against the cover's edge.
    let (x0, x1) = (x0.max(x_min).min(x_max), x1.min(x_max).max(x_min));
    let (y0, y1) = (y0.max(y_min).min(y_max), y1.min(y_max).max(y_min));
    let (x1, y1) = (x1.max(x0), y1.max(y0));
    *start = AnnotationPoint { x: x0, y: y0 };
    *end = AnnotationPoint { x: x1, y: y1 };
  }
  out
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
