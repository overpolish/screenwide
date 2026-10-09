// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The join from one chunk's restored voice to the next. Sidon speaks each
//! chunk afresh, so two chunks' takes on the same moment seldom line up wave
//! for wave, and blending them plays both at once, which is heard as a
//! doubled voice. The overlap is only there so the new chunk hears what came
//! before; the join is a cut, at the quietest moment late in the overlap,
//! softened over a few milliseconds.

#[cfg(test)]
mod tests;

/// Output samples the cut is softened over: 20 ms at 48 kHz.
const SOFTEN: usize = 960;
/// How far apart the moments tried for the cut are: 10 ms.
const STEP: usize = 480;

/// Puts `held`, what the chunk before made of the overlap, in place of the
/// start of `voice` up to the cut.
pub(super) fn splice(held: &[f32], voice: &mut [f32]) {
  let shared = held.len().min(voice.len());
  let soften = SOFTEN.min(shared);
  let cut = quietest(&held[..shared], &voice[..shared], soften);
  voice[..cut].copy_from_slice(&held[..cut]);
  for at in 0..soften {
    #[expect(clippy::cast_precision_loss, reason = "at most SOFTEN")]
    let rising = 0.5 - 0.5 * (std::f32::consts::PI * (at as f32 + 0.5) / soften as f32).cos();
    voice[cut + at] = held[cut + at] * (1.0 - rising) + voice[cut + at] * rising;
  }
}

/// Where the softened cut starts: the moment in the later half of the
/// overlap, by which the new chunk has heard enough to settle, where the
/// louder of the two takes is quietest, so neither has much to lose there.
fn quietest(held: &[f32], voice: &[f32], soften: usize) -> usize {
  let last = held.len() - soften;
  let energy = |take: &[f32], at: usize| take[at..at + soften].iter().map(|s| s * s).sum::<f32>();
  (last / 2..=last)
    .step_by(STEP)
    .min_by(|&a, &b| {
      let loudness = |at| energy(held, at).max(energy(voice, at));
      loudness(a).total_cmp(&loudness(b))
    })
    .unwrap_or(last)
}
