// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::fit::fit_similarity;
use super::super::flow::track;
use super::super::geometry::{distance, Similarity};
use super::super::luma::Pyramid;
use super::super::search::Template;
use super::{
  seed, Observation, Status, Tracker, AGREEMENT, FRAME_SCALE, MIN_INLIERS, POINTS, REFERENCE_SCALE,
  ROUND_TRIP, TEMPLATE_EVERY, THRESHOLD,
};

impl Tracker {
  pub(super) fn follow(&mut self, next: &Pyramid) -> Observation {
    let [vx, vy] = self.velocity;
    let inset = self.inset();
    let mut from = Vec::with_capacity(self.points.len());
    let mut to = Vec::with_capacity(self.points.len());
    for &point in &self.points {
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

    self.points = match &frame_fit {
      Some(fit) => to
        .iter()
        .zip(&fit.inliers)
        .filter_map(|(point, &inlier)| inlier.then_some(*point))
        .collect(),
      None => reference.map(|(_, found)| found).unwrap_or_default(),
    };
    let before = self.transform.apply(anchor);
    let after = transform.apply(anchor);
    self.velocity = [after[0] - before[0], after[1] - before[1]];
    self.transform = transform;

    let region = transform.apply_rect(&self.target.region);
    if self.points.len() < POINTS / 2 {
      let fresh = seed(
        &next.levels[0],
        &self.target,
        &region,
        POINTS - self.points.len(),
        &self.points,
      );
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
