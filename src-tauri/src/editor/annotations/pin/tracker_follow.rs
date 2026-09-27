// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::fit::{fit_similarity, Fit};
use super::super::flow::track;
use super::super::geometry::{distance, Similarity};
use super::super::luma::Pyramid;
use super::super::search::Template;
use super::{
  seed, Observation, Status, Tracker, AGREEMENT, FRAME_SCALE, MIN_INLIERS, POINTS, REFERENCE_SCALE,
  ROUND_TRIP, TEMPLATE_EVERY, THRESHOLD,
};

/// How many times more points the target was last confirmed with than an
/// unconfirmed answer may rest on.
const COLLAPSE: usize = 3;

impl Tracker {
  pub(super) fn follow(&mut self, next: &Pyramid) -> Observation {
    let [vx, vy] = self.velocity;
    let inset = self.inset();
    let mut from = Vec::with_capacity(self.points.len());
    let mut to = Vec::with_capacity(self.points.len());
    let mut vetted = Vec::with_capacity(self.points.len());
    for (&point, &was_vetted) in self.points.iter().zip(&self.vetted) {
      let guess = [point[0] + vx, point[1] + vy];
      if !inset.contains(guess) {
        continue;
      }
      let Some(found) =
        track(&self.prev, next, point, guess).filter(|found| inset.contains(*found))
      else {
        continue;
      };
      let Some(back) = track(next, &self.prev, found, point) else {
        continue;
      };
      if distance(back, point) <= ROUND_TRIP {
        from.push(point);
        to.push(found);
        vetted.push(was_vetted);
      }
    }
    let frame_fit = fit_similarity(&from, &to, THRESHOLD, FRAME_SCALE, &mut self.rng)
      .filter(|fit| fit.count >= MIN_INLIERS && fit.count * 5 >= from.len() * 2);
    let candidate = frame_fit
      .as_ref()
      .map(|fit| self.transform.then(&fit.transform));
    let prior = candidate.unwrap_or_else(|| self.transform.shifted(self.velocity));
    let reference = self.match_reference(next, &prior);

    let anchor = self.target.anchor;
    let confirmed = matches!(
      (&reference, candidate),
      (Some((pinned, _)), Some(followed))
        if distance(pinned.apply(anchor), followed.apply(anchor)) <= AGREEMENT
    );
    // Followed without the pinned look confirming it, an answer is only as
    // good as the points behind it. Most of them gone at once, or the rest
    // split between two movements, is the target going under something that
    // stays put - a sticky header, a window moved over it - with the points
    // left on what covers it: the answer would follow the cover.
    let candidate = match (&frame_fit, candidate) {
      (Some(fit), Some(followed)) if !confirmed => (!self.collapsed(fit, &vetted)
        && !self.split(&from, &to, fit, &followed))
      .then_some(followed),
      (_, candidate) => candidate,
    };
    if let (true, Some(fit)) = (confirmed, &frame_fit) {
      self.supported = fit.count;
    }
    let chosen = match (&reference, candidate) {
      (Some((pinned, _)), Some(followed))
        if distance(pinned.apply(anchor), followed.apply(anchor)) <= AGREEMENT =>
      {
        Some((*pinned, 1.0))
      }
      (Some((pinned, _)), None) => Some((*pinned, 0.7)),
      (_, Some(followed)) => {
        // Followed from the frame before, with most of its points agreeing:
        // sound, even where the content no longer looks as it did when it
        // was pinned. Fewer agreeing than the fit's floor is no answer at
        // all, so a followed frame never falls to doubtful; how many agree
        // only decides how far smoothing trusts it.
        let fit = frame_fit.as_ref().expect("a followed answer has a fit");
        let share = fit.count as f32 / from.len().max(1) as f32;
        let support = (fit.count as f32 / 20.0).min(1.0);
        Some((followed, 0.55 + 0.35 * share * support))
      }
      (None, None) => None,
    };
    let Some((transform, confidence)) = chosen else {
      return self.lose(next.ms);
    };

    // A point is vetted once it has moved with an answer the pinned look
    // confirmed; one taken since and not yet confirmed may sit on a cover.
    (self.points, self.vetted) = match &frame_fit {
      Some(fit) => to
        .iter()
        .zip(&fit.inliers)
        .zip(&vetted)
        .filter(|((_, &inlier), _)| inlier)
        .map(|((point, _), &was_vetted)| (*point, confirmed || was_vetted))
        .unzip(),
      None => {
        let found = reference.map(|(_, found)| found).unwrap_or_default();
        let vetted = vec![true; found.len()];
        (found, vetted)
      }
    };
    let before = self.transform.apply(anchor);
    let after = transform.apply(anchor);
    self.velocity = [after[0] - before[0], after[1] - before[1]];
    self.transform = transform;

    let region = transform.apply_rect(&self.target.region);
    // Fresh points are only taken where the pinned look confirms the target
    // is. Unconfirmed, the region may already be over whatever is covering
    // it, and points taken there would carry the answer onto the cover; the
    // points followed since the last confirmation dwindle instead, and the
    // target is given up once too few are left.
    if confirmed && self.points.len() < POINTS / 2 {
      let fresh = seed(
        &next.levels[0],
        &self.target,
        &region,
        POINTS - self.points.len(),
        &self.points,
      );
      self.vetted.extend(std::iter::repeat_n(false, fresh.len()));
      self.points.extend(fresh);
    }
    if self.points.len() < MIN_INLIERS {
      return self.lose(next.ms);
    }
    if confidence >= 0.7 {
      self.since_template += 1;
      if self.since_template >= TEMPLATE_EVERY {
        if let Some(template) = Template::cut(next, &region) {
          self.recent_template = Some(template);
          self.since_template = 0;
        }
      }
    }
    Observation {
      ms: next.ms,
      transform,
      confidence,
      status: Status::Tracked,
    }
  }

  /// Whether an answer rests on too few vetted points, against how many
  /// agreed when the target was last confirmed, to be the target still.
  fn collapsed(&self, fit: &Fit, vetted: &[bool]) -> bool {
    let kept = fit
      .inliers
      .iter()
      .zip(vetted)
      .filter(|(&inlier, &vetted)| inlier && vetted)
      .count();
    kept * COLLAPSE < self.supported
  }

  /// Whether the points `fit` left out agree on a movement of their own,
  /// nearly as well supported and taking the anchor somewhere else.
  fn split(
    &mut self,
    from: &[[f32; 2]],
    to: &[[f32; 2]],
    fit: &Fit,
    followed: &Similarity,
  ) -> bool {
    let (rest_from, rest_to): (Vec<_>, Vec<_>) = from
      .iter()
      .zip(to)
      .zip(&fit.inliers)
      .filter(|(_, &inlier)| !inlier)
      .map(|((from, to), _)| (*from, *to))
      .unzip();
    if rest_from.len() < MIN_INLIERS {
      return false;
    }
    let Some(other) = fit_similarity(&rest_from, &rest_to, THRESHOLD, FRAME_SCALE, &mut self.rng)
    else {
      return false;
    };
    let anchor = self.target.anchor;
    other.count >= MIN_INLIERS
      && other.count * 2 >= fit.count
      && distance(
        self.transform.then(&other.transform).apply(anchor),
        followed.apply(anchor),
      ) > AGREEMENT
  }

  /// The pinned frame's points found in `next` from where `prior` puts
  /// them: the movement they agree on and where the agreeing ones landed.
  pub(super) fn match_reference(
    &mut self,
    next: &Pyramid,
    prior: &Similarity,
  ) -> Option<(Similarity, Vec<[f32; 2]>)> {
    let inset = self.inset();
    let mut tried = 0;
    let mut from = Vec::with_capacity(self.reference_points.len());
    let mut to = Vec::with_capacity(self.reference_points.len());
    for &point in &self.reference_points {
      let guess = prior.apply(point);
      if !inset.contains(guess) {
        continue;
      }
      tried += 1;
      if let Some(found) =
        track(&self.reference, next, point, guess).filter(|found| inset.contains(*found))
      {
        from.push(point);
        to.push(found);
      }
    }
    let fit = fit_similarity(&from, &to, 1.5 * THRESHOLD, REFERENCE_SCALE, &mut self.rng)?;
    if fit.count < MIN_INLIERS || fit.count * 2 < tried {
      return None;
    }
    let found = to
      .iter()
      .zip(&fit.inliers)
      .filter_map(|(point, &inlier)| inlier.then_some(*point))
      .collect();
    Some((fit.transform, found))
  }
}
