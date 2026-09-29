// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One spotlight handing its light to the next.
//!
//! Two spotlights butted up on the timeline are one light moving rather than
//! two lights: the first does not fade out and the second does not fade in,
//! so the shade holds, and the light glides from the first box to the second
//! over the second's first moments. The glide travels as a stroke draws in:
//! paced by how far it goes as a path of that length is, on the same eased
//! curve. A clip that starts within a frame of another's end, on the same
//! track and the same side of the camera, takes the light from it. Each clip
//! hands on to one clip and takes from one, so where several end and start
//! together the nearest boxes pair up, and the rest fade as they would
//! alone. Spotlights showing side by side are two lights and never join. The
//! preview and both exports resolve clips through here, so they hand over
//! alike.

use super::reveal::{spotlight_reveal_window, SpotlightJoins};
use crate::editor::annotations::pace::travel_ms;
use crate::editor::annotations::reveal::{eased_travel, REVEAL_PHASE_SHARE};
use crate::editor::annotations::timing::{placed_annotation, RecordingAnnotationClip};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

/// How far apart one clip's end and the next's start may be, in source
/// milliseconds, and still join: a frame at thirty frames a second.
const JOIN_SLACK_MS: u64 = 34;

/// The clips a spotlight clip takes its light from and hands it on to, by
/// their places in the clip list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpotlightLinks {
  pub(crate) from: Option<usize>,
  pub(crate) onward: Option<usize>,
}

impl SpotlightLinks {
  pub(crate) fn joins(self) -> SpotlightJoins {
    SpotlightJoins {
      from: self.from.is_some(),
      onward: self.onward.is_some(),
    }
  }
}

fn is_spotlight(clip: &RecordingAnnotationClip) -> bool {
  clip.annotation.animated && matches!(clip.annotation.shape, AnnotationShape::Spotlight { .. })
}

/// The spotlight as it is last drawn in `clip`, which the next one glides
/// from.
fn last_drawn(clip: &RecordingAnnotationClip) -> Option<Annotation> {
  placed_annotation(clip, clip.end_ms.saturating_sub(1)).map(|(annotation, _)| annotation)
}

fn centre(annotation: &Annotation) -> Option<[f64; 2]> {
  match annotation.shape {
    AnnotationShape::Spotlight { start, end } => {
      Some([(start.x + end.x) * 0.5, (start.y + end.y) * 0.5])
    }
    _ => None,
  }
}

/// Every clip's links: none for a clip that is not a spotlight, or has no
/// spotlight butted up against it.
pub(crate) fn spotlight_links(clips: &[RecordingAnnotationClip]) -> Vec<SpotlightLinks> {
  let mut links = vec![SpotlightLinks::default(); clips.len()];
  let mut pairs = Vec::new();
  for (before, first) in clips.iter().enumerate() {
    let Some(from) = is_spotlight(first)
      .then(|| last_drawn(first))
      .flatten()
      .as_ref()
      .and_then(centre)
    else {
      continue;
    };
    for (after, second) in clips.iter().enumerate() {
      let joins = after != before
        && is_spotlight(second)
        && second.track_id == first.track_id
        && second.annotation.above_camera == first.annotation.above_camera
        && second.start_ms > first.start_ms
        && second.start_ms.abs_diff(first.end_ms) <= JOIN_SLACK_MS;
      if !joins {
        continue;
      }
      let Some(to) = placed_annotation(second, second.start_ms)
        .as_ref()
        .and_then(|(annotation, _)| centre(annotation))
      else {
        continue;
      };
      pairs.push(((from[0] - to[0]).hypot(from[1] - to[1]), before, after));
    }
  }
  pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then((a.1, a.2).cmp(&(b.1, b.2))));
  for (_, before, after) in pairs {
    if links[before].onward.is_none() && links[after].from.is_none() {
      links[before].onward = Some(after);
      links[after].from = Some(before);
    }
  }
  links
}

/// Where clip `index` stops being drawn, in source time: its end, or where
/// the clip it hands on to starts, so a frame's gap between them never
/// shows the picture without its shade.
pub(crate) fn drawn_until(
  clips: &[RecordingAnnotationClip],
  links: &[SpotlightLinks],
  index: usize,
) -> u64 {
  links[index]
    .onward
    .map_or(clips[index].end_ms, |next| clips[next].start_ms)
}

fn mix(from: f64, to: f64, progress: f64) -> f64 {
  from + (to - from) * progress
}

fn mix_point(from: AnnotationPoint, to: AnnotationPoint, progress: f64) -> AnnotationPoint {
  AnnotationPoint {
    x: mix(from.x, to.x, progress),
    y: mix(from.y, to.y, progress),
  }
}

/// A spotlight's box as its low and high corners. The drag that drew it may
/// have set out from any corner, and pairing drag corners across two boxes
/// drawn different ways folds the light through nothing on its way.
fn corners(annotation: &Annotation) -> Option<[AnnotationPoint; 2]> {
  match annotation.shape {
    AnnotationShape::Spotlight { start, end } => Some([
      AnnotationPoint {
        x: start.x.min(end.x),
        y: start.y.min(end.y),
      },
      AnnotationPoint {
        x: start.x.max(end.x),
        y: start.y.max(end.y),
      },
    ]),
    _ => None,
  }
}

/// How far the farther corner of the box travels from `from` to `to`, in
/// source pixels, so a box that grows moves as far as its moving corner.
fn travel(from: &Annotation, to: &Annotation) -> f64 {
  match (corners(from), corners(to)) {
    (Some([a, b]), Some([c, d])) => (c.x - a.x)
      .hypot(c.y - a.y)
      .max((d.x - b.x).hypot(d.y - b.y)),
    _ => 0.0,
  }
}

/// The box clip `index` takes the light from, and how long it takes to glide
/// from there to its own in output time, on a picture `frame` source pixels
/// in size: as long as a path as long as the glide takes to draw in, and
/// never more of its clip's `duration_ms` than an arrival may take. `None`
/// for a clip that takes no light over.
fn glide(
  clips: &[RecordingAnnotationClip],
  links: &[SpotlightLinks],
  index: usize,
  frame: (u32, u32),
  duration_ms: f32,
) -> Option<(Annotation, f32)> {
  let from = last_drawn(&clips[links[index].from?])?;
  let clip = &clips[index];
  let (to, _) = placed_annotation(clip, clip.start_ms)?;
  let ms = travel_ms(travel(&from, &to), frame).min(duration_ms * REVEAL_PHASE_SHARE);
  (ms > 0.0).then_some((from, ms))
}

/// How long clip `index` glides in over, as [`glide`] works it out.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn glide_ms(
  clips: &[RecordingAnnotationClip],
  links: &[SpotlightLinks],
  index: usize,
  frame: (u32, u32),
  duration_ms: f32,
) -> Option<f32> {
  glide(clips, links, index, frame, duration_ms).map(|(_, ms)| ms)
}

/// `to` as it is drawn `progress` of its glide's time from `from`: its box,
/// its rounding and its softness between the two, and how far its blur has
/// arrived where only one of them blurs.
fn glided(from: &Annotation, to: &Annotation, progress: f32) -> (Annotation, f32) {
  let eased = eased_travel(progress);
  let along = f64::from(eased);
  let mut drawn = to.clone();
  if let (Some([a, b]), Some([c, d])) = (corners(from), corners(to)) {
    drawn.shape = AnnotationShape::Spotlight {
      start: mix_point(a, c, along),
      end: mix_point(b, d, along),
    };
  }
  drawn.style.radius = mix(from.style.radius, to.style.radius, along);
  drawn.style.softness = mix(from.style.softness, to.style.softness, along);
  drawn.style.blur = from.style.blur || to.style.blur;
  let blur_share = match (from.style.blur, to.style.blur) {
    (true, false) => 1.0 - eased,
    (false, true) => eased,
    _ => 1.0,
  };
  (drawn, blur_share)
}

/// Spotlight clip `index` of `clips`, drawn as `annotation` `elapsed_ms`
/// into the `duration_ms` its reveal plays over in output time, on a picture
/// `frame` source pixels in size: held at its joins, and while it is still
/// gliding in, between the box it took the light from and its own.
pub(crate) fn joined(
  clips: &[RecordingAnnotationClip],
  links: &[SpotlightLinks],
  index: usize,
  frame: (u32, u32),
  annotation: Annotation,
  elapsed_ms: f32,
  duration_ms: f32,
) -> (Annotation, f32) {
  let (mut drawn, blur_share) = match glide(clips, links, index, frame, duration_ms) {
    Some((from, ms)) if elapsed_ms < ms => glided(&from, &annotation, elapsed_ms / ms),
    _ => (annotation, 1.0),
  };
  drawn.reveal = spotlight_reveal_window(elapsed_ms, duration_ms, links[index].joins(), blur_share);
  (drawn, blur_share)
}

#[cfg(test)]
#[path = "handoff_tests.rs"]
mod tests;
