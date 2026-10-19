// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the face is in the camera picture over the recording, smoothed into
//! the path a scene that follows it centres on. The face is found a few times
//! a second after recording, so the path is worked out from the whole of it:
//! the crop holds still while the face stays near where it has been sitting
//! and glides once it settles somewhere new, starting as the person moves
//! rather than after, the way a camera operator who knows the shot would.
//! Detector noise and small movements never move it. Where the face is not
//! found, the path holds where it last was.

/// The path's resolution in camera milliseconds: the camera's own rate.
const STEP_MS: f64 = 1_000.0 / 30.0;
/// How long a run of misses is bridged by sliding from one sighting to the
/// next; past it the path holds the last sighting until the next.
const BRIDGE_MS: f64 = 500.0;
/// Detector noise, smoothed away first, in milliseconds.
const NOISE_SIGMA_MS: f64 = 250.0;
/// How far the face may wander from where it has been sitting before the
/// crop follows, as a share of the face's own height.
const STILL_SHARE: f64 = 0.1;
/// How long the crop takes to ease into and out of a move, in milliseconds.
const EASE_SIGMA_MS: f64 = 500.0;

/// The face seen in one camera frame: its centre as shares of the picture's
/// width and height, and its height as a share of the picture's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceSighting {
  pub ms: f64,
  pub x: f64,
  pub y: f64,
  pub size: f64,
}

/// The smoothed centre of the face, as shares of the camera picture, a step
/// apart from the camera's start.
#[derive(Clone, Debug, PartialEq)]
pub struct FacePath {
  points: Vec<(f64, f64)>,
}

impl FacePath {
  /// The path through `sightings`, in time order, of a camera picture
  /// `aspect` wide to its height that runs `duration_ms`. `None` where the
  /// face was never found.
  pub fn new(sightings: &[FaceSighting], aspect: f64, duration_ms: f64) -> Option<Self> {
    let first = sightings.first()?;
    let last_ms = sightings
      .last()
      .map_or(0.0, |last| last.ms)
      .max(duration_ms);
    let count = (last_ms / STEP_MS).ceil() as usize + 1;
    let mut xs = Vec::with_capacity(count);
    let mut ys = Vec::with_capacity(count);
    let mut next = 0;
    let mut held = *first;
    for step in 0..count {
      let ms = step as f64 * STEP_MS;
      while next < sightings.len() && sightings[next].ms <= ms {
        held = sightings[next];
        next += 1;
      }
      let (x, y) = match sightings.get(next) {
        Some(ahead) if next > 0 && ahead.ms - held.ms <= BRIDGE_MS => {
          let share = (ms - held.ms) / (ahead.ms - held.ms).max(f64::EPSILON);
          (
            held.x + (ahead.x - held.x) * share,
            held.y + (ahead.y - held.y) * share,
          )
        }
        _ => (held.x, held.y),
      };
      xs.push(x);
      ys.push(y);
    }
    let mut sizes = sightings
      .iter()
      .map(|sighting| sighting.size)
      .collect::<Vec<_>>();
    sizes.sort_by(f64::total_cmp);
    let size = sizes[sizes.len() / 2];
    let zone = (
      STILL_SHARE * size / aspect.max(f64::EPSILON),
      STILL_SHARE * size,
    );
    let (xs, ys) = (smoothed(&xs, NOISE_SIGMA_MS), smoothed(&ys, NOISE_SIGMA_MS));
    let (xs, ys) = still_stretches(&xs, &ys, zone);
    let (xs, ys) = (smoothed(&xs, EASE_SIGMA_MS), smoothed(&ys, EASE_SIGMA_MS));
    Some(Self {
      points: xs.into_iter().zip(ys).collect(),
    })
  }

  /// Where the face is centred `ms` into the camera, held at either end.
  pub fn at(&self, ms: f64) -> (f64, f64) {
    let at = (ms / STEP_MS).max(0.0);
    let below = (at.floor() as usize).min(self.points.len() - 1);
    let above = (below + 1).min(self.points.len() - 1);
    let share = (at - below as f64).clamp(0.0, 1.0);
    let (a, b) = (self.points[below], self.points[above]);
    (a.0 + (b.0 - a.0) * share, a.1 + (b.1 - a.1) * share)
  }
}

/// `values` blurred over `sigma_ms`, the ends held.
fn smoothed(values: &[f64], sigma_ms: f64) -> Vec<f64> {
  let sigma = sigma_ms / STEP_MS;
  let radius = (3.0 * sigma).ceil() as isize;
  let weights = (-radius..=radius)
    .map(|offset| (-0.5 * (offset as f64 / sigma).powi(2)).exp())
    .collect::<Vec<_>>();
  let total: f64 = weights.iter().sum();
  let last = values.len() as isize - 1;
  (0..values.len() as isize)
    .map(|index| {
      weights
        .iter()
        .zip(-radius..=radius)
        .map(|(weight, offset)| weight * values[(index + offset).clamp(0, last) as usize])
        .sum::<f64>()
        / total
    })
    .collect()
}

/// The path split into stretches where it stays within `zone` of the
/// stretch's own average, each held at that average, so the crop sits on
/// where the face is while it is still rather than at the edge of a slack
/// zone, and moves once for each real move.
fn still_stretches(xs: &[f64], ys: &[f64], zone: (f64, f64)) -> (Vec<f64>, Vec<f64>) {
  let mut held_x = vec![0.0; xs.len()];
  let mut held_y = vec![0.0; ys.len()];
  let (mut start, mut sum_x, mut sum_y) = (0, 0.0, 0.0);
  let mut hold = |from: usize, to: usize, sum_x: f64, sum_y: f64| {
    let count = (to - from) as f64;
    held_x[from..to].fill(sum_x / count);
    held_y[from..to].fill(sum_y / count);
  };
  for (index, (&x, &y)) in xs.iter().zip(ys).enumerate() {
    let count = (index - start) as f64;
    if count > 0.0 && ((x - sum_x / count).abs() > zone.0 || (y - sum_y / count).abs() > zone.1) {
      hold(start, index, sum_x, sum_y);
      (start, sum_x, sum_y) = (index, 0.0, 0.0);
    }
    sum_x += x;
    sum_y += y;
  }
  hold(start, xs.len(), sum_x, sum_y);
  (held_x, held_y)
}

#[cfg(test)]
#[path = "face_tests.rs"]
mod tests;
