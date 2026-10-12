// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::geometry::{distance, Rect, Similarity};
use super::super::luma::Pyramid;
use super::super::search::Template;
use super::{seed, Observation, Status, Tracker, MAX_SEARCH, MIN_INLIERS, POINTS};

/// How closely a lost target's latest look must match to be taken back
/// without the pinned frame's points agreeing.
const UNVERIFIED_SCORE: f32 = 0.92;

/// How far past twice its own size a lost target is looked for, in tracking
/// pixels.
const SEARCH_SLACK: f32 = 32.0;
/// How long a target found again by the hand counts as having been lost, in
/// frames, which widens the search to about a hundred tracking pixels more.
const FIND_LOST_FRAMES: u32 = 25;
impl Tracker {
  /// Looks for the target near where it was last seen, by its latest look
  /// and then by the pinned one. A find is taken once the pinned frame's
  /// points agree with it. A target lost on screen whose look has changed
  /// too far for them is also taken back on a near-exact match of its latest
  /// look; one that went off the edge never is, since anything else on the
  /// frame could be the best match while the target itself is still off it.
  pub(super) fn reacquire(&mut self, next: &Pyramid) -> Observation {
    let templates = [
      (self.recent_template.clone(), false),
      (self.reference_template.clone(), true),
    ];
    self.look_for(next, &templates)
  }

  /// Looks for the target by each of `templates` in turn, each marked with
  /// whether it is the pinned look, as [`Tracker::reacquire`] does.
  fn look_for(&mut self, next: &Pyramid, templates: &[(Option<Template>, bool)]) -> Observation {
    self.lost_frames += 1;
    let last = self.transform;
    let centre = self.target.region.centre();
    let around = last.apply(centre);
    let frame = Rect {
      x0: 0.0,
      y0: 0.0,
      x1: next.width(),
      y1: next.height(),
    };
    let diagonal = (frame.width().powi(2) + frame.height().powi(2)).sqrt();
    let size = (self.target.region.width().powi(2) + self.target.region.height().powi(2)).sqrt()
      * last.scale();
    // Something covered on screen may move on behind what covers it; a
    // target off the edge comes back in where it left.
    let growth = if self.status == Status::Lost {
      4.0 * self.lost_frames as f32
    } else {
      0.0
    };
    let radius = (2.0 * size + SEARCH_SLACK + growth).min(MAX_SEARCH * diagonal);
    // A search costs about ten frames of following, so a target off the
    // frame is looked for every other frame: it comes back at most a frame
    // late, while its entrance plays.
    if self.status == Status::Hidden && self.lost_frames.is_multiple_of(2) {
      return Observation {
        ms: next.ms,
        transform: last,
        confidence: 0.0,
        status: self.status,
      };
    }
    for (template, pinned) in templates {
      let pinned = *pinned;
      let Some(template) = template else {
        continue;
      };
      let Some((found, score)) = template.find(next, around, radius) else {
        continue;
      };
      let candidate = last.shifted([found[0] - around[0], found[1] - around[1]]);
      let accepted = match self.match_reference(next, &candidate) {
        Some((transform, _))
          if distance(transform.apply(centre), found) <= 0.25 * size.max(8.0) =>
        {
          Some((transform, 0.8))
        }
        _ if !pinned && self.status == Status::Lost && score >= UNVERIFIED_SCORE => {
          Some((candidate, 0.4))
        }
        _ => None,
      };
      let Some((transform, confidence)) = accepted else {
        continue;
      };
      let region = transform.apply_rect(&self.target.region);
      let points = seed(&next.levels[0], &self.target, &region, POINTS, &[]);
      if points.len() < MIN_INLIERS {
        continue;
      }
      self.supported = points.len();
      self.vetted = vec![true; points.len()];
      self.points = points;
      self.transform = transform;
      self.velocity = [0.0; 2];
      self.status = Status::Tracked;
      self.lost_frames = 0;
      return Observation {
        ms: next.ms,
        transform,
        confidence,
        status: Status::Tracked,
      };
    }
    Observation {
      ms: next.ms,
      transform: last,
      confidence: 0.0,
      status: self.status,
    }
  }

  /// Looks for the target on `next` alone, around `around` - a movement from
  /// the pinned frame, in tracking pixels - as widely as for a target lost a
  /// while, by a finer cut of the pinned look, and takes it only where the
  /// pinned frame's points agree. For content the hand says is back in view,
  /// where nothing was followed to get there.
  pub(crate) fn find(&mut self, next: &Pyramid, around: [f32; 2]) -> Option<Observation> {
    self.transform = Similarity::IDENTITY.shifted(around);
    self.velocity = [0.0; 2];
    self.status = Status::Lost;
    self.lost_frames = FIND_LOST_FRAMES;
    let fine = Template::cut_fine(&self.reference, &self.target.region);
    let observation = self.look_for(next, &[(fine, true)]);
    (observation.status == Status::Tracked && observation.confidence >= 0.8).then_some(observation)
  }
}
