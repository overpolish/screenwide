// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An edit made where a pin had drawn its annotation, written back into the
//! clip: as a keyframe where it says where the content is, and into the
//! annotation itself where it changes the annotation.

use super::model::KEYFRAME_SLACK_MS;
use super::resolve::{displaced, placement, Placement, NO_COVER};
use super::target::anchor_of;
use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

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

/// A redaction's box as `[x0, y0, x1, y1]`.
fn redact_box(shape: &AnnotationShape) -> Option<[f64; 4]> {
  let AnnotationShape::Redact { start, end, .. } = shape else {
    return None;
  };
  Some([
    start.x.min(end.x),
    start.y.min(end.y),
    start.x.max(end.x),
    start.y.max(end.y),
  ])
}

/// `shape` with a redaction's box set to `[x0, y0, x1, y1]`, kept the right
/// way round. Other kinds are left as they are.
fn boxed(shape: &AnnotationShape, [x0, y0, x1, y1]: [f64; 4]) -> AnnotationShape {
  let mut out = shape.clone();
  if let AnnotationShape::Redact { start, end, .. } = &mut out {
    *start = AnnotationPoint {
      x: x0.min(x1),
      y: y0.min(y1),
    };
    *end = AnnotationPoint {
      x: x0.max(x1),
      y: y0.max(y1),
    };
  }
  out
}

/// What the hand left of a redaction shown grown: each edge it dragged
/// where it was dropped, and each edge it left alone without its growth,
/// since the growth belongs to that moment and not to the box.
fn settled(shown: &AnnotationShape, grown: [f64; 4], ungrown: [f64; 4]) -> AnnotationShape {
  let Some(left) = redact_box(shown) else {
    return shown.clone();
  };
  let sides: [f64; 4] = std::array::from_fn(|side| {
    if (left[side] - grown[side]).abs() < 1e-6 {
      ungrown[side]
    } else {
      left[side]
    }
  });
  boxed(shown, sides)
}

/// `shape` with a redaction's box drawn in by `sides` - left, top, right,
/// bottom. Other kinds are left as they are.
fn drawn_in(shape: &AnnotationShape, [left, top, right, bottom]: [f64; 4]) -> AnnotationShape {
  match redact_box(shape) {
    Some([x0, y0, x1, y1]) => boxed(shape, [x0 + left, y0 + top, x1 - right, y1 - bottom]),
    None => shape.clone(),
  }
}

/// Writes `shown` - the annotation as an edit left it at `ms`, where its pin
/// had drawn it - back into `clip`.
///
/// Moving the whole annotation says where its content is at that moment, so
/// it becomes a keyframe there and the geometry it was drawn with stays.
/// Resizing a redaction away from the frame it was pinned on says how big
/// its content is at that moment, so that is a keyframe too. Anything else -
/// a redaction resized on its pinned frame, a moved grip, new text - changes
/// the annotation itself, so it is written back to where it was drawn.
///
/// A redaction is shown grown over doubtful frames; the growth belongs to
/// that moment, not to the box, so it comes off what the hand left first.
pub(crate) fn fold(clip: &mut RecordingAnnotationClip, shown: &Annotation, ms: u64) {
  let Some(pin) = clip.pin.as_ref() else {
    clip.annotation = shown.clone();
    return;
  };
  let place = placement(clip, pin, ms);
  let pinned_ms = pin.pinned_ms;
  let kept = Annotation {
    shape: clip.annotation.shape.clone(),
    ..shown.clone()
  };
  // Moving takes no part in size: the keyframe keeps any resize it has.
  if let Some([dx, dy]) = movement(&displaced(&clip.annotation, &place).shape, &shown.shape) {
    if dx != 0.0 || dy != 0.0 {
      if let Some(pin) = clip.pin.as_mut() {
        pin.set_keyframe(ms, [place.offset[0] + dx, place.offset[1] + dy], None);
      }
    }
    clip.annotation = kept;
    return;
  }
  // What the hand left is measured against the box untrimmed: a trim, like
  // growth, belongs to the moment and comes off the edges left alone.
  let plain = Placement {
    edges: [0.0; 4],
    grow: [0.0; 4],
    view: NO_COVER,
    ..place
  };
  let drawn = displaced(&clip.annotation, &plain);
  let sized = Placement {
    grow: [0.0; 4],
    view: NO_COVER,
    ..place
  };
  let ungrown = match (
    redact_box(&displaced(&clip.annotation, &place).shape),
    redact_box(&displaced(&clip.annotation, &sized).shape),
  ) {
    (Some(grown), Some(ungrown)) => settled(&shown.shape, grown, ungrown),
    _ => shown.shape.clone(),
  };
  if ms.abs_diff(pinned_ms) > KEYFRAME_SLACK_MS {
    if let (Some(from), Some(to)) = (redact_box(&drawn.shape), redact_box(&ungrown)) {
      let edges = [
        from[0] - to[0],
        from[1] - to[1],
        to[2] - from[2],
        to[3] - from[3],
      ];
      if let Some(pin) = clip.pin.as_mut() {
        pin.set_keyframe(ms, place.offset, Some(edges));
      }
      clip.annotation = kept;
      return;
    }
  }
  // Back to where it was drawn, about the anchor the placement moved.
  let edited = drawn_in(&ungrown, place.edges);
  let [ax, ay] = anchor_of(&drawn);
  let scale = if redact_box(&shown.shape).is_some() {
    place.scale.max(1e-3)
  } else {
    1.0
  };
  let [dx, dy] = place.offset;
  let (bx, by) = (ax - dx, ay - dy);
  clip.annotation = Annotation {
    shape: edited.mapped(|point| AnnotationPoint {
      x: bx + (point.x - ax) / scale,
      y: by + (point.y - ay) / scale,
    }),
    ..shown.clone()
  };
}
