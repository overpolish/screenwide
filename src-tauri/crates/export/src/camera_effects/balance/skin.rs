// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the person's skin says about the light. Skin of every tone sits near
//! one hue on the vectorscope, so how far it drifts from that hue shows the
//! light's colour along green to magenta, and whether an estimate leaves it
//! looking like skin.

use super::super::look::{cbcr, LUMA, SKIN_HUE};
use super::{axes, displayed, from_axes, saturation, usable, Sample, BEHIND, INNER};

impl Sample {
  /// The median linear colour of the person's pixels that look like skin
  /// once `gains` are applied, or nothing with too little skin to tell.
  fn skin(&self, gains: [f64; 3]) -> Option<[f64; 3]> {
    let mut channels: [Vec<f32>; 3] = Default::default();
    for (lin, place) in self.linear.iter().zip(&self.places) {
      if place & INNER != 0 && looks_like_skin(displayed(lin.map(f64::from), gains)) {
        for (channel, value) in channels.iter_mut().zip(lin) {
          channel.push(*value);
        }
      }
    }
    if !self.enough(channels[0].len(), 200.0) {
      return None;
    }
    Some(channels.map(|mut channel| {
      let middle = channel.len() / 2;
      f64::from(*channel.select_nth_unstable_by(middle, f32::total_cmp).1)
    }))
  }

  /// Gains, from `start` and along green to magenta only, that put the
  /// person's skin on the skin line, the smallest change that does it.
  fn skin_tint(&self, start: [f64; 3], limit: f64) -> Option<[f64; 3]> {
    let mut gains = start;
    for _ in 0..3 {
      let skin = self.skin(gains)?;
      let mut best = gains;
      let mut best_cost = f64::INFINITY;
      for step in 0..=40 {
        let tint = -limit + 2.0 * limit * f64::from(step) / 40.0;
        let shift = [-tint / 2.0, tint, -tint / 2.0];
        let candidate = [0, 1, 2].map(|index| gains[index] * shift[index].exp());
        let cost = off_line(displayed(skin, candidate))
          + 0.15 * shift.iter().map(|step| step * step).sum::<f64>();
        if cost < best_cost {
          (best, best_cost) = (candidate, cost);
        }
      }
      gains = best.map(|gain| gain / best[1]);
    }
    Some(gains)
  }

  /// Warm against cool from the `room`, held to a bound unless much of the
  /// room is plainly grey once corrected; green against magenta from the
  /// skin, or from the room where there is too little skin.
  pub(super) fn combined(&self, room: [f64; 3]) -> [f64; 3] {
    let (temperature, room_tint) = axes(room);
    let behind = self.count(&|_, place| place & BEHIND != 0);
    let calm = self.count(&|lin, place| {
      let corrected = [0, 1, 2].map(|index| (f64::from(lin[index]) * room[index]) as f32);
      place & BEHIND != 0 && saturation(lin, room) < 0.18 && usable(corrected)
    });
    let bound = if calm as f64 > 0.35 * behind.max(1) as f64 {
      0.45
    } else {
      0.25
    };
    let temperature = temperature.clamp(-bound, bound);
    let tint = match self.skin_tint(from_axes(temperature, 0.0), 0.12) {
      Some(skin) => axes(skin).1,
      None => room_tint.clamp(-0.06, 0.06),
    };
    from_axes(temperature, tint.clamp(-0.12, 0.12))
  }

  /// Of the `candidates`, and of leaving the colour alone, the one that
  /// leaves the person's skin nearest its line, a smaller change winning a
  /// close call. One that leaves skin with less than 85% of its colour is
  /// turned away. Without skin to judge by, half of gray-edge's way.
  pub(super) fn judged(&self, candidates: &[Option<[f64; 3]>]) -> [f64; 3] {
    let Some(skin) = self.skin([1.0; 3]) else {
      return candidates[0].map_or([1.0; 3], |edge| edge.map(f64::sqrt));
    };
    let chroma = |gains| {
      let [cb, cr] = cbcr(displayed(skin, gains));
      cb.hypot(cr)
    };
    let before = chroma([1.0; 3]);
    let mut best = [1.0; 3];
    let mut best_cost = f64::INFINITY;
    for gains in std::iter::once([1.0; 3]).chain(candidates.iter().flatten().copied()) {
      if chroma(gains) < 0.85 * before {
        continue;
      }
      let Some(seen) = self.skin(gains) else {
        continue;
      };
      let cost = off_line(displayed(seen, gains)).to_degrees()
        + 20.0 * gains.iter().map(|gain| gain.ln().powi(2)).sum::<f64>();
      if cost < best_cost {
        (best, best_cost) = (gains, cost);
      }
    }
    best
  }
}

/// How far a display colour's hue is from the skin line, in radians.
fn off_line(rgb: [f32; 3]) -> f64 {
  let [cb, cr] = cbcr(rgb);
  f64::from((cr.atan2(cb) - SKIN_HUE).abs())
}

/// Mid brightness, a hue near the skin line and a modest chroma: narrow
/// enough that a red jacket or a pink top beside the face is not read as
/// skin, and by hue alone, so any skin tone qualifies.
fn looks_like_skin(rgb: [f32; 3]) -> bool {
  let [cb, cr] = cbcr(rgb);
  let chroma = cb.hypot(cr);
  let luma = rgb
    .iter()
    .zip(LUMA)
    .map(|(value, weight)| value * weight)
    .sum::<f32>();
  (cr.atan2(cb) - SKIN_HUE).abs() < 15f32.to_radians()
    && chroma > 0.02
    && chroma < 0.15
    && luma > 0.08
    && luma < 0.9
}
