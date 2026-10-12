// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Forward-backward (Rauch-Tung-Striebel) smoothing of a raw path under a
//! constant velocity model, one axis at a time.
//!
//! Each measurement's variance comes from how well its frame matched. A clean
//! match measures to a hundredth of a pixel squared, well under what the
//! model lets the motion change by in a frame, so screen content passes
//! through unchanged: a scroll that stops dead stops dead. Weak matches on
//! moving footage are pulled toward the motion around them, and a stretch
//! with no measurement at all is bridged from both of its ends.

/// Time is measured in frames of a 60 fps recording, which the tuning below
/// is expressed in.
const FRAME_MS: f32 = 1000.0 / 60.0;

/// How one axis is modelled: how far its motion may change per frame, as an
/// acceleration variance, and how far a perfect and a doubtful measurement
/// are trusted.
#[derive(Clone, Copy)]
pub(crate) struct Tuning {
  pub(crate) acceleration: f32,
  pub(crate) exact: f32,
  pub(crate) doubtful: f32,
}

/// Position in source pixels, where a 5K recording has about two to a point.
pub(crate) const POSITION: Tuning = Tuning {
  acceleration: 4.0,
  doubtful: 2.0,
  exact: 1e-6,
};
/// The natural logarithm of the change of size.
pub(crate) const LOG_SCALE: Tuning = Tuning {
  acceleration: 1e-5,
  exact: 4e-6,
  doubtful: 1e-4,
};

impl Tuning {
  /// The variance of a measurement made with `confidence` from 0 to 1.
  fn variance(&self, confidence: f32) -> f32 {
    let confidence = confidence.clamp(0.05, 1.0);
    self.exact + self.doubtful * (1.0 / (confidence * confidence) - 1.0)
  }
}

/// One sample: its time, and the measured value with its confidence where
/// the frame was measured.
#[derive(Clone, Copy)]
pub(crate) struct Sample {
  pub(crate) ms: u64,
  pub(crate) value: Option<(f32, f32)>,
}

type State = [f32; 2];
/// A symmetric 2 by 2 covariance: `[pp, pv, vv]`.
type Covariance = [f32; 3];

fn predict(state: State, covariance: Covariance, dt: f32, q: f32) -> (State, Covariance) {
  let [pp, pv, vv] = covariance;
  let state = [state[0] + dt * state[1], state[1]];
  let (dt2, dt3, dt4) = (dt * dt, dt * dt * dt, dt * dt * dt * dt);
  let covariance = [
    pp + 2.0 * dt * pv + dt2 * vv + 0.25 * dt4 * q,
    pv + dt * vv + 0.5 * dt3 * q,
    vv + dt2 * q,
  ];
  (state, covariance)
}

/// The smoothed value at every sample. Before the first measurement and
/// after the last there is nothing to bridge from, so those samples hold the
/// nearest measured value instead of running on at the last speed.
pub(crate) fn smooth(samples: &[Sample], tuning: Tuning) -> Vec<f32> {
  let Some(first) = samples.iter().position(|sample| sample.value.is_some()) else {
    return vec![0.0; samples.len()];
  };
  let last = samples
    .iter()
    .rposition(|sample| sample.value.is_some())
    .expect("a first measurement means a last one");
  let span = &samples[first..=last];

  let mut predicted: Vec<(State, Covariance)> = Vec::with_capacity(span.len());
  let mut filtered: Vec<(State, Covariance)> = Vec::with_capacity(span.len());
  let (initial, confidence) = span[0].value.expect("the span starts measured");
  let mut state = [initial, 0.0];
  let mut covariance = [tuning.variance(confidence), 0.0, 100.0];
  for (index, sample) in span.iter().enumerate() {
    if index > 0 {
      let dt = (sample.ms.saturating_sub(span[index - 1].ms) as f32 / FRAME_MS).max(0.01);
      (state, covariance) = predict(state, covariance, dt, tuning.acceleration);
    }
    predicted.push((state, covariance));
    if let (Some((value, confidence)), true) = (sample.value, index > 0) {
      let r = tuning.variance(confidence);
      let [pp, pv, vv] = covariance;
      let innovation = value - state[0];
      let total = pp + r;
      let (kp, kv) = (pp / total, pv / total);
      state = [state[0] + kp * innovation, state[1] + kv * innovation];
      covariance = [(1.0 - kp) * pp, (1.0 - kp) * pv, vv - kv * pv];
    }
    filtered.push((state, covariance));
  }

  let mut smoothed = vec![[0.0_f32; 2]; span.len()];
  smoothed[span.len() - 1] = filtered[span.len() - 1].0;
  for index in (0..span.len() - 1).rev() {
    let (state, [pp, pv, vv]) = filtered[index];
    let (next_state, [npp, npv, nvv]) = predicted[index + 1];
    let dt = (span[index + 1].ms.saturating_sub(span[index].ms) as f32 / FRAME_MS).max(0.01);
    // C = P F^T (P_next)^-1, with F = [[1, dt], [0, 1]].
    let (a, b, c, d) = (pp + dt * pv, pv, pv + dt * vv, vv);
    let determinant = npp * nvv - npv * npv;
    if determinant.abs() <= f32::EPSILON {
      smoothed[index] = state;
      continue;
    }
    let (ipp, ipv, ivv) = (nvv / determinant, -npv / determinant, npp / determinant);
    let gain = [
      [a * ipp + b * ipv, a * ipv + b * ivv],
      [c * ipp + d * ipv, c * ipv + d * ivv],
    ];
    let difference = [
      smoothed[index + 1][0] - next_state[0],
      smoothed[index + 1][1] - next_state[1],
    ];
    smoothed[index] = [
      state[0] + gain[0][0] * difference[0] + gain[0][1] * difference[1],
      state[1] + gain[1][0] * difference[0] + gain[1][1] * difference[1],
    ];
  }

  let mut out = Vec::with_capacity(samples.len());
  out.extend(std::iter::repeat_n(smoothed[0][0], first));
  out.extend(smoothed.iter().map(|state| state[0]));
  out.extend(std::iter::repeat_n(
    smoothed[span.len() - 1][0],
    samples.len() - last - 1,
  ));
  out
}
