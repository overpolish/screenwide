// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a pinned annotation is drawn at a moment, and how an edit made to it
//! there is written back into its clip.

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
  /// The stretches the annotation shows for, `[start, end)` in source
  /// milliseconds.
  pub(crate) shown: Vec<[u64; 2]>,
  /// The stretches it shows for but its content was lost in, bridged from
  /// either side.
  pub(crate) weak: Vec<[u64; 2]>,
  /// The stretches its content was off the frame.
  pub(crate) hidden: Vec<[u64; 2]>,
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
}

/// Where a pinned annotation is drawn at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
  /// How far it has moved from where it was drawn, in source pixels.
  pub(crate) offset: [f64; 2],
  /// How much a redaction's box has grown with its content.
  pub(crate) scale: f64,
  pub(crate) grow: [f64; 4],
  /// The stretch it is showing through, which its arrival and leaving play
  /// over; `None` while it is hidden.
  pub(crate) shown: Option<[u64; 2]>,
}

/// Where `clip`'s pinned annotation is at `ms`.
///
/// On a keyframe it is exactly where the keyframe puts it, whatever the path,
/// so an annotation dragged there stays under the hand while its new path is
/// worked out. Elsewhere it follows the path it has, which may still be the
/// path of the keyframes before a change; with no path yet it waits at the
/// nearest keyframe.
pub(crate) fn placement(clip: &RecordingAnnotationClip, pin: &AnnotationPin, ms: u64) -> Placement {
  let whole = [clip.start_ms, clip.end_ms];
  let path = pin.path.as_deref();
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
      grow: [0.0; 4],
      shown: path.and_then(shown).or(Some(whole)),
    };
  }
  if let Some((path, index)) = path.and_then(|path| Some((path, path.index(ms)?))) {
    let sample = &path.samples[index];
    return Placement {
      offset: [f64::from(sample.dx), f64::from(sample.dy)],
      scale: f64::from(sample.scale),
      grow: path
        .growth
        .get(index)
        .map_or([0.0; 4], |grow| grow.map(f64::from)),
      shown: shown(path),
    };
  }
  let offset = pin
    .nearest(ms)
    .map_or([0.0; 2], |keyframe| [keyframe.dx, keyframe.dy]);
  Placement {
    offset,
    scale: 1.0,
    grow: [0.0; 4],
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
  let span = clip.annotation.shape.kind().reveal_span_ms();
  clip.end_ms.saturating_sub(from) as f32 >= span
}

/// `annotation` where `placement` puts it. A redaction's box also grows and
/// shrinks with its content and is grown over doubtful frames; the other
/// kinds keep their size, since they are drawn at the canvas's scale.
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
    let [left, top, right, bottom] = placement.grow;
    let (x0, x1) = (start.x.min(end.x) - left, start.x.max(end.x) + right);
    let (y0, y1) = (start.y.min(end.y) - top, start.y.max(end.y) + bottom);
    *start = AnnotationPoint { x: x0, y: y0 };
    *end = AnnotationPoint { x: x1, y: y1 };
  }
  out
}

/// How far `from` was moved to become `to`, when moving is all that was
/// done to it.
fn movement(from: &AnnotationShape, to: &AnnotationShape) -> Option<[f64; 2]> {
  const EPSILON: f64 = 1e-6;
  let flat = |shape: &AnnotationShape| shape.mapped(|_| AnnotationPoint::default());
  if flat(from) != flat(to) {
    return None;
  }
  let (a, b) = (from.points(), to.points());
  let delta = [b[0].x - a[0].x, b[0].y - a[0].y];
  a.iter()
    .zip(&b)
    .all(|(a, b)| (b.x - a.x - delta[0]).abs() < EPSILON && (b.y - a.y - delta[1]).abs() < EPSILON)
    .then_some(delta)
}

/// Writes `shown` - the annotation as an edit left it at `ms`, where its pin
/// had drawn it - back into `clip`.
///
/// Moving the whole annotation says where its content is at that moment, so
/// it becomes a keyframe there and the geometry it was drawn with stays.
/// Anything else - a resized box, a moved grip, new text - changes the
/// annotation itself, so it is written back to where it was drawn.
pub(crate) fn fold(clip: &mut RecordingAnnotationClip, shown: &Annotation, ms: u64) {
  let Some(pin) = clip.pin.as_ref() else {
    clip.annotation = shown.clone();
    return;
  };
  let place = placement(clip, pin, ms);
  let expected = displaced(&clip.annotation, &place);
  match movement(&expected.shape, &shown.shape) {
    Some([dx, dy]) => {
      let kept = clip.annotation.shape.clone();
      if dx != 0.0 || dy != 0.0 {
        if let Some(pin) = clip.pin.as_mut() {
          pin.set_keyframe(ms, [place.offset[0] + dx, place.offset[1] + dy]);
        }
      }
      clip.annotation = Annotation {
        shape: kept,
        ..shown.clone()
      };
    }
    None => {
      let [ax, ay] = anchor_of(&expected);
      let redaction = matches!(shown.shape, AnnotationShape::Redact { .. });
      let scale = if redaction {
        place.scale.max(1e-3)
      } else {
        1.0
      };
      let [dx, dy] = place.offset;
      // The anchor where the annotation was drawn, before the placement moved
      // it.
      let (bx, by) = (ax - dx, ay - dy);
      clip.annotation = Annotation {
        shape: shown.shape.mapped(|point| AnnotationPoint {
          x: bx + (point.x - ax) / scale,
          y: by + (point.y - ay) / scale,
        }),
        ..shown.clone()
      };
    }
  }
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
