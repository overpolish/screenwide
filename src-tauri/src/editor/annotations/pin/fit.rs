// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{distance, Rng, Similarity};

/// How many pairs of points are tried as the movement they agree on.
const HYPOTHESES: usize = 48;
/// Two points closer than this say too little about a change of size.
const MIN_SPREAD: f32 = 4.0;

/// The movement most of the pairs agree on.
pub(crate) struct Fit {
  pub(crate) transform: Similarity,
  pub(crate) inliers: Vec<bool>,
  pub(crate) count: usize,
}

/// The least-squares similarity carrying `from` onto `to`, over the pairs
/// `keep` allows. `None` with fewer than two pairs or with every point on
/// top of the others.
fn least_squares(
  from: &[[f32; 2]],
  to: &[[f32; 2]],
  keep: &dyn Fn(usize) -> bool,
) -> Option<Similarity> {
  let (mut count, mut fx, mut fy, mut tx, mut ty) = (0.0_f32, 0.0, 0.0, 0.0, 0.0);
  for (index, (f, t)) in from.iter().zip(to).enumerate() {
    if keep(index) {
      count += 1.0;
      fx += f[0];
      fy += f[1];
      tx += t[0];
      ty += t[1];
    }
  }
  if count < 2.0 {
    return None;
  }
  let (fx, fy, tx, ty) = (fx / count, fy / count, tx / count, ty / count);
  let (mut dot, mut cross, mut norm) = (0.0_f32, 0.0, 0.0);
  for (index, (f, t)) in from.iter().zip(to).enumerate() {
    if keep(index) {
      let (px, py) = (f[0] - fx, f[1] - fy);
      let (qx, qy) = (t[0] - tx, t[1] - ty);
      dot += px * qx + py * qy;
      cross += px * qy - py * qx;
      norm += px * px + py * py;
    }
  }
  if norm <= f32::EPSILON {
    return None;
  }
  let (a, b) = (dot / norm, cross / norm);
  Some(Similarity {
    a,
    b,
    tx: tx - (a * fx - b * fy),
    ty: ty - (b * fx + a * fy),
  })
}

fn inliers(
  transform: &Similarity,
  from: &[[f32; 2]],
  to: &[[f32; 2]],
  threshold: f32,
) -> (Vec<bool>, usize) {
  let flags: Vec<bool> = from
    .iter()
    .zip(to)
    .map(|(f, t)| distance(transform.apply(*f), *t) <= threshold)
    .collect();
  let count = flags.iter().filter(|&&inlier| inlier).count();
  (flags, count)
}

/// The similarity most pairs agree on to within `threshold` pixels, refined
/// over those pairs. A change of size beyond `max_scale` either way is not a
/// movement anything on a screen makes between the two frames, so no
/// hypothesis proposing one is counted.
pub(crate) fn fit_similarity(
  from: &[[f32; 2]],
  to: &[[f32; 2]],
  threshold: f32,
  max_scale: f32,
  rng: &mut Rng,
) -> Option<Fit> {
  let n = from.len().min(to.len());
  if n < 2 {
    return None;
  }
  let (from, to) = (&from[..n], &to[..n]);
  let plausible = |transform: &Similarity| {
    let scale = transform.scale();
    scale.is_finite() && scale <= max_scale && scale >= 1.0 / max_scale
  };
  let mut best: Option<(Similarity, usize)> = None;
  for _ in 0..HYPOTHESES {
    let i = rng.below(n);
    let j = rng.below(n);
    if i == j || distance(from[i], from[j]) < MIN_SPREAD {
      continue;
    }
    let Some(hypothesis) = least_squares(from, to, &|index| index == i || index == j) else {
      continue;
    };
    if !plausible(&hypothesis) {
      continue;
    }
    let (_, count) = inliers(&hypothesis, from, to, threshold);
    if best.is_none_or(|(_, best_count)| count > best_count) {
      best = Some((hypothesis, count));
    }
  }
  // Every point clustered together: a pure movement is still measurable.
  let (hypothesis, _) = best.or_else(|| {
    let (dx, dy) = from.iter().zip(to).fold((0.0, 0.0), |(dx, dy), (f, t)| {
      (dx + t[0] - f[0], dy + t[1] - f[1])
    });
    let shift = Similarity::IDENTITY.shifted([dx / n as f32, dy / n as f32]);
    Some((shift, 0))
  })?;
  let (flags, _) = inliers(&hypothesis, from, to, threshold);
  let transform = least_squares(from, to, &|index| flags[index])
    .filter(plausible)
    .unwrap_or(hypothesis);
  let (inliers, count) = inliers(&transform, from, to, threshold);
  Some(Fit {
    transform,
    inliers,
    count,
  })
}
