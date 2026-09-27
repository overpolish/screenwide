// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::cover::Covers;
use super::features::corners;
use super::geometry::{Rect, Rng, Similarity};
use super::luma::{Plane, Pyramid};
use super::search::Template;

/// Following the target from one frame to the next.
#[path = "tracker_follow.rs"]
mod follow;
/// Finding the target again once it is lost.
#[path = "tracker_reacquire.rs"]
mod reacquire;

/// How many points are followed from frame to frame.
const POINTS: usize = 64;
/// How many of the pinned frame's points each frame is also matched against.
const REFERENCE_POINTS: usize = 40;
/// The fewest agreeing points a movement is taken from.
const MIN_INLIERS: usize = 5;
/// How close two followed points may be, in tracking pixels.
const MIN_DISTANCE: f32 = 5.0;
/// How far a point may land from where it started once followed back, in
/// tracking pixels. Further, it slid along an edge or onto something else.
const ROUND_TRIP: f32 = 0.7;
/// How far a point may sit from the agreed movement and still agree.
const THRESHOLD: f32 = 1.0;
/// The largest change of size between two frames, either way.
const FRAME_SCALE: f32 = 1.15;
/// The largest change of size from the pinned frame, either way.
const REFERENCE_SCALE: f32 = 4.0;
/// How far apart, in tracking pixels, the pinned frame's answer and the
/// frame-to-frame answer may be and still be one answer.
const AGREEMENT: f32 = 2.0;
/// How often a well-followed target's look is taken again, in frames, for
/// finding it once lost after it has turned or changed.
const TEMPLATE_EVERY: u32 = 15;
/// The share of the region left on the frame below which losing the target
/// means it went off the edge rather than out of sight on screen.
const MIN_VISIBLE: f32 = 0.35;
/// How far inside the frame a followed point must stay, in tracking pixels:
/// a flow window reaching past the edge reads the edge smeared outward and
/// drags its point along.
const EDGE_MARGIN: f32 = 9.0;
/// The widest search for a lost target, as a share of the frame's diagonal.
const MAX_SEARCH: f32 = 0.35;
/// A region with fewer corners than this is grown, up to [`GROWTHS`] times,
/// before tracking starts.
const MIN_CORNERS: usize = 12;
const GROWTHS: usize = 3;

/// What is followed: a box on the pinned frame and the point whose place on
/// the frame decides whether the annotation shows, in tracking pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Target {
  pub(crate) region: Rect,
  pub(crate) anchor: [f32; 2],
  pub(crate) centre_weighted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Status {
  Tracked,
  /// Out of sight while it should still be on screen: covered, turned or
  /// changed past recognition.
  Lost,
  /// Gone off the edge of the frame.
  Hidden,
}

/// One frame's answer: how the pinned frame maps onto it, and how far that
/// is to be trusted. A frame that is not tracked carries the last answer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Observation {
  pub(crate) ms: u64,
  pub(crate) transform: Similarity,
  pub(crate) confidence: f32,
  pub(crate) status: Status,
}

pub(crate) struct Tracker {
  target: Target,
  reference: Pyramid,
  reference_points: Vec<[f32; 2]>,
  reference_template: Option<Template>,
  recent_template: Option<Template>,
  since_template: u32,
  prev: Pyramid,
  points: Vec<[f32; 2]>,
  /// Beside each of `points`, whether it has moved with an answer the pinned
  /// look confirmed.
  vetted: Vec<bool>,
  transform: Similarity,
  velocity: [f32; 2],
  status: Status,
  lost_frames: u32,
  /// How many points agreed the last time the pinned look confirmed the
  /// target, which an unconfirmed answer is measured against.
  supported: usize,
  covers: Covers,
  rng: Rng,
}

/// How far the size may change between two frames, as a share, for the
/// frame to vote on covers: the lines of content growing do not all move
/// alike.
const STEADY_SCALE: f32 = 0.01;

/// The central share of a centre-weighted box its points are taken from
/// while it has enough of them there.
const CORE: f32 = 0.6;

/// Up to `max` corners to follow in `region`, which is `target`'s region
/// where it now is. A centre-weighted box takes them from its core first.
fn seed(
  plane: &Plane,
  target: &Target,
  region: &Rect,
  max: usize,
  existing: &[[f32; 2]],
) -> Vec<[f32; 2]> {
  if target.centre_weighted {
    let core = corners(
      plane,
      &region.scaled(CORE),
      max,
      MIN_DISTANCE,
      true,
      existing,
    );
    if core.len() >= MIN_CORNERS.min(max) {
      return core;
    }
  }
  corners(
    plane,
    region,
    max,
    MIN_DISTANCE,
    target.centre_weighted,
    existing,
  )
}

impl Tracker {
  /// Starts from the pinned frame, growing a region too plain to follow
  /// until it has enough corners.
  pub(crate) fn new(reference: Pyramid, mut target: Target) -> Self {
    let plane = &reference.levels[0];
    let mut points = Vec::new();
    for growth in 0..=GROWTHS {
      points = seed(plane, &target, &target.region, POINTS, &[]);
      if points.len() >= MIN_CORNERS || growth == GROWTHS {
        break;
      }
      target.region = target.region.scaled(1.5);
    }
    let reference_points = points.iter().copied().take(REFERENCE_POINTS).collect();
    let reference_template = Template::cut(&reference, &target.region);
    let points_count = points.len();
    let covers = Covers::new(plane.width, plane.height);
    Self {
      target,
      reference_points,
      // The pinned look is searched for anyway; a later one is kept once
      // the target has been followed a while.
      recent_template: None,
      reference_template,
      since_template: 0,
      prev: reference.clone(),
      reference,
      points,
      vetted: vec![true; points_count],
      transform: Similarity::IDENTITY,
      velocity: [0.0; 2],
      status: Status::Tracked,
      lost_frames: 0,
      supported: points_count,
      covers,
      rng: Rng::new(0x9e37_79b9),
    }
  }

  /// The pinned frame's own answer.
  pub(crate) fn pinned(&self) -> Observation {
    Observation {
      ms: self.reference.ms,
      transform: Similarity::IDENTITY,
      confidence: 1.0,
      status: if self.points.len() >= MIN_INLIERS {
        Status::Tracked
      } else {
        Status::Lost
      },
    }
  }

  pub(crate) fn step(&mut self, next: Pyramid) -> Observation {
    let (was, before) = (self.status, self.transform);
    let observation = if self.status == Status::Tracked && self.points.len() >= MIN_INLIERS {
      self.follow(&next)
    } else {
      self.reacquire(&next)
    };
    // Only a movement followed from one frame to the next says how each
    // line of the content moved.
    let after = observation.transform;
    if was == Status::Tracked
      && observation.status == Status::Tracked
      && (after.scale() / before.scale() - 1.0).abs() < STEADY_SCALE
    {
      let centre = self.target.region.centre();
      let ([x0, y0], [x1, y1]) = (before.apply(centre), after.apply(centre));
      let region = after.apply_rect(&self.target.region);
      self.covers.watch(
        &self.prev.levels[0],
        &next.levels[0],
        &region,
        [x1 - x0, y1 - y0],
      );
    }
    self.prev = next;
    observation
  }

  /// The part of the frame no still cover lies over, in tracking pixels:
  /// infinite on each side without one.
  pub(crate) fn view(&self) -> Rect {
    self.covers.view()
  }

  fn frame(&self) -> Rect {
    Rect {
      x0: 0.0,
      y0: 0.0,
      x1: self.prev.width(),
      y1: self.prev.height(),
    }
  }

  /// The frame less the margin a flow window needs to lie wholly on it.
  fn inset(&self) -> Rect {
    let frame = self.frame();
    Rect {
      x0: frame.x0 + EDGE_MARGIN,
      y0: frame.y0 + EDGE_MARGIN,
      x1: frame.x1 - EDGE_MARGIN,
      y1: frame.y1 - EDGE_MARGIN,
    }
  }

  /// Gives the target up in the frame at `ms`: as gone off the edge where
  /// its anchor would be off the frame, too little of it would be left on
  /// it, or it was crossing an edge outward; as out of sight otherwise.
  fn lose(&mut self, ms: u64) -> Observation {
    let moved = self.transform.shifted(self.velocity);
    let region = moved.apply_rect(&self.target.region);
    let frame = self.frame();
    let visible = region.intersect(&frame).area() / region.area().max(1.0);
    let [vx, vy] = self.velocity;
    let leaving = (region.x0 < frame.x0 && vx < 0.0)
      || (region.x1 > frame.x1 && vx > 0.0)
      || (region.y0 < frame.y0 && vy < 0.0)
      || (region.y1 > frame.y1 && vy > 0.0);
    self.status =
      if visible < MIN_VISIBLE || leaving || !frame.contains(moved.apply(self.target.anchor)) {
        Status::Hidden
      } else {
        Status::Lost
      };
    self.lost_frames = 0;
    self.points.clear();
    self.vetted.clear();
    Observation {
      ms,
      transform: self.transform,
      confidence: 0.0,
      status: self.status,
    }
  }
}
