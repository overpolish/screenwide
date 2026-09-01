// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// A contact episode longer than this is a press or a glide, not a tap. Frame
/// timestamps arrive in seconds.
const TAP_MAX_DURATION: f64 = 0.25;
/// How far the two-finger centroid may drift, in normalised 0..1 trackpad
/// units, before the episode counts as a scroll rather than a tap.
const TAP_MAX_TRAVEL: f32 = 0.05;

pub(super) struct TwoFingerTap {
  #[allow(dead_code)]
  pub(super) timestamp: f64,
}

/// One contact episode: everything between the trackpad reporting its first
/// contact and reporting none again.
struct Episode {
  started_at: f64,
  peak_touches: usize,
  /// The first two-finger centroid, which later ones are measured against.
  origin: Option<(f32, f32)>,
  travel: f32,
}

/// Turns frames of contact counts and centroids into two-finger taps. Pure
/// state, so the gesture can be tested without a trackpad.
#[derive(Default)]
pub(super) struct TapRecognizer {
  episode: Option<Episode>,
}

impl TapRecognizer {
  /// Folds one frame in, and returns the tap if the episode just closed as one.
  pub(super) fn update(
    &mut self,
    num_touches: usize,
    centroid: Option<(f32, f32)>,
    timestamp: f64,
  ) -> Option<TwoFingerTap> {
    if num_touches > 0 {
      let episode = self.episode.get_or_insert(Episode {
        started_at: timestamp,
        peak_touches: 0,
        origin: None,
        travel: 0.0,
      });
      episode.peak_touches = episode.peak_touches.max(num_touches);
      // Only two-finger frames steer the travel test; the finger that lifts
      // first would otherwise drag the centroid across the whole trackpad.
      if num_touches == 2 {
        if let Some(point) = centroid {
          match episode.origin {
            Some(origin) => {
              let travel = (point.0 - origin.0).hypot(point.1 - origin.1);
              episode.travel = episode.travel.max(travel);
            }
            None => episode.origin = Some(point),
          }
        }
      }
      return None;
    }

    let episode = self.episode.take()?;
    (episode.peak_touches == 2
      && timestamp - episode.started_at < TAP_MAX_DURATION
      && episode.travel < TAP_MAX_TRAVEL)
      .then_some(TwoFingerTap { timestamp })
  }
}

#[cfg(test)]
mod tests {
  use super::TapRecognizer;

  /// Feeds a run of frames and returns whether they closed exactly one tap.
  fn taps(frames: &[(usize, Option<(f32, f32)>, f64)]) -> bool {
    let mut recognizer = TapRecognizer::default();
    frames
      .iter()
      .filter_map(|(count, centroid, timestamp)| recognizer.update(*count, *centroid, *timestamp))
      .count()
      == 1
  }

  #[test]
  fn recognises_a_clean_two_finger_tap() {
    assert!(taps(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.502, 0.501)), 0.04),
      (0, None, 0.08),
    ]));
  }

  #[test]
  fn ignores_a_three_finger_tap() {
    assert!(!taps(&[
      (2, Some((0.5, 0.5)), 0.0),
      (3, Some((0.5, 0.5)), 0.01),
      (0, None, 0.06),
    ]));
  }

  #[test]
  fn ignores_a_single_finger_tap() {
    assert!(!taps(&[(1, Some((0.5, 0.5)), 0.0), (0, None, 0.05)]));
  }

  #[test]
  fn ignores_a_slow_two_finger_press() {
    assert!(!taps(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.5, 0.5)), 0.2),
      (0, None, 0.4),
    ]));
  }

  #[test]
  fn ignores_a_short_two_finger_scroll() {
    assert!(!taps(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.5, 0.6)), 0.05),
      (0, None, 0.1),
    ]));
  }

  #[test]
  fn recognises_a_tap_whose_fingers_lift_one_at_a_time() {
    assert!(taps(&[
      (2, Some((0.4, 0.4)), 0.0),
      (1, Some((0.9, 0.1)), 0.05),
      (0, None, 0.09),
    ]));
  }
}
