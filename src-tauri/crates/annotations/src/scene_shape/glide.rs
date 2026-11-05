// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How an annotation travels to its place in a scene and back. It sets off
//! as the scene arrives, as the picture does, but takes as long as the
//! distance asks: a short move as long as the scene's own arrival, a long
//! one longer, up to a limit, so a move across the picture is never a rush.
//! It leaves early enough to be back where it belongs as the scene ends.

use screenwide_scenes::RecordingSceneClip;
use screenwide_timeline::easing::ease_in_out_cubic;

use super::{AnnotationShape, RecordingAnnotationClip};
use crate::timing::{covers, progress, reaches_video_ends, SCENE_TRANSITION_MS};
use crate::AnnotationPoint;

/// The longest a glide takes, in output milliseconds.
const LONGEST_GLIDE_MS: f32 = 1_500.0;
/// The share of the picture's diagonal a glide crosses in the scene's own
/// arrival time; farther ones take longer, as the square root of the
/// distance, so long moves are calm without short ones dragging.
const BRISK_SHARE: f64 = 0.25;

/// Where a frame falls among the scenes: `timeline_ms` into a video
/// `video_ms` long, drawn on a picture `picture` source pixels in size.
#[derive(Clone, Copy, Debug)]
pub struct SceneMoment {
  pub timeline_ms: u64,
  pub video_ms: Option<u64>,
  pub picture: (u32, u32),
}

/// How far `from`'s geometry is from `to`'s, in source pixels: the farthest
/// any of its points goes, and an image's corner as it grows. Points that
/// cannot travel, such as a stroke of another length, count by its ends.
fn travel(from: &AnnotationShape, to: &AnnotationShape) -> f64 {
  use AnnotationShape as S;
  let farthest = |pairs: &[(AnnotationPoint, AnnotationPoint)]| {
    pairs
      .iter()
      .map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
      .fold(0.0, f64::max)
  };
  match (from, to) {
    (
      S::Arrow {
        start: s0,
        control: c0,
        end: e0,
      },
      S::Arrow {
        start: s1,
        control: c1,
        end: e1,
      },
    ) => farthest(&[(*s0, *s1), (*c0, *c1), (*e0, *e1)]),
    (S::Counter { center: a, .. }, S::Counter { center: b, .. })
    | (S::Text { origin: a, .. }, S::Text { origin: b, .. }) => farthest(&[(*a, *b)]),
    (
      S::Redact {
        start: s0, end: e0, ..
      },
      S::Redact {
        start: s1, end: e1, ..
      },
    )
    | (
      S::Shape {
        start: s0, end: e0, ..
      },
      S::Shape {
        start: s1, end: e1, ..
      },
    )
    | (S::Spotlight { start: s0, end: e0 }, S::Spotlight { start: s1, end: e1 })
    | (S::Focus { start: s0, end: e0 }, S::Focus { start: s1, end: e1 })
    | (
      S::Highlight {
        start: s0, end: e0, ..
      },
      S::Highlight {
        start: s1, end: e1, ..
      },
    )
    | (
      S::Magnify {
        start: s0, end: e0, ..
      },
      S::Magnify {
        start: s1, end: e1, ..
      },
    ) => farthest(&[(*s0, *s1), (*e0, *e1)]),
    (S::Draw { points: p0, .. }, S::Draw { points: p1, .. }) if p0.len() == p1.len() => p0
      .iter()
      .zip(p1)
      .map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
      .fold(0.0, f64::max),
    (S::Draw { points: p0, .. }, S::Draw { points: p1, .. }) => {
      match (p0.first().zip(p0.last()), p1.first().zip(p1.last())) {
        (Some((a0, a1)), Some((b0, b1))) => farthest(&[(*a0, *b0), (*a1, *b1)]),
        _ => 0.0,
      }
    }
    (
      S::Image {
        center: c0,
        size: z0,
        ..
      },
      S::Image {
        center: c1,
        size: z1,
        ..
      },
    ) => (c1.x - c0.x).hypot(c1.y - c0.y) + (z1 - z0).abs() / 2.0,
    _ => 0.0,
  }
}

/// How long a glide from `from` to `to` takes on `picture`, in ms. Its twin
/// is `glideMs` in `recording-annotation-glides.ts`, which marks glides on
/// the strip.
pub(super) fn glide_ms(
  from: &AnnotationShape,
  to: &AnnotationShape,
  (width, height): (u32, u32),
) -> f32 {
  let diagonal = f64::from(width).hypot(f64::from(height));
  if diagonal <= 0.0 {
    return SCENE_TRANSITION_MS;
  }
  let share = travel(from, to) / diagonal;
  (SCENE_TRANSITION_MS * (share / BRISK_SHARE).sqrt() as f32)
    .clamp(SCENE_TRANSITION_MS, LONGEST_GLIDE_MS)
}
fn ease(share: f32) -> f64 {
  f64::from(ease_in_out_cubic(share.clamp(0.0, 1.0)))
}

impl RecordingAnnotationClip {
  /// The shape the annotation is drawn with at `at` among `scenes`: in a
  /// scene where it has a place, there, having glided from where it was as
  /// the scene arrived if it was showing then; leaving, on its way back if it
  /// still shows after. `None` where it keeps its own. A pinned annotation
  /// follows its content and keeps its own.
  pub fn shape_in_scenes(
    &self,
    scenes: &[RecordingSceneClip],
    at: SceneMoment,
  ) -> Option<AnnotationShape> {
    if self.scene_shapes.is_empty() || self.pin.is_some() {
      return None;
    }
    let index = scenes
      .iter()
      .position(|scene| covers(at.video_ms, [scene.start_ms, scene.end_ms], at.timeline_ms))?;
    let scene = &scenes[index];
    let own = &self.annotation.shape;
    let here = self.shape_in_scene(&scene.id);
    let before = index
      .checked_sub(1)
      .map(|previous| &scenes[previous])
      .filter(|previous| previous.end_ms == scene.start_ms);
    let before_place = before.and_then(|previous| self.shape_in_scene(&previous.id));
    if here.is_none() && before_place.is_none() {
      return None;
    }
    let (elapsed, length) = progress([scene.start_ms, scene.end_ms], at.timeline_ms)?;
    let (starts_video, ends_video) =
      reaches_video_ends(at.video_ms, [scene.start_ms, scene.end_ms]);
    // Each glide keeps to its half of the scene, or all of it where the
    // scene holds at an end of the video.
    let room = if starts_video || ends_video {
      length
    } else {
      length / 2.0
    };
    let to = here.as_ref().unwrap_or(own);
    // Only an annotation already showing as the scene arrives has anywhere
    // to glide from; one that appears inside it appears in its place there.
    if !starts_video && self.start_ms < scene.start_ms {
      let from = before_place.as_ref().unwrap_or(own);
      let window = glide_ms(from, to, at.picture).min(room);
      if elapsed < window && from != to {
        return Some(from.toward(to, ease(elapsed / window)));
      }
    }
    let handed_on = scenes
      .get(index + 1)
      .is_some_and(|next| next.start_ms == scene.end_ms);
    // Gone before the scene ends, it leaves from its place there.
    let stays_after = self.end_ms > scene.end_ms;
    if let Some(place) = here
      .as_ref()
      .filter(|_| !handed_on && !ends_video && stays_after)
    {
      let window = glide_ms(place, own, at.picture).min(room);
      if length - elapsed < window {
        return Some(own.toward(place, ease((length - elapsed) / window)));
      }
    }
    here
  }
}
