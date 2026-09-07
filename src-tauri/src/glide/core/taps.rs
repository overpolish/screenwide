// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
const TAP_MAX_DURATION: f64 = 0.25;
const TAP_MAX_TRAVEL: f32 = 0.05;

pub(crate) struct TwoFingerTap;

struct Episode {
  started_at: f64,
  peak_touches: usize,
  origin: Option<(f32, f32)>,
  travel: f32,
}

#[derive(Default)]
pub(crate) struct TapRecognizer {
  episode: Option<Episode>,
}

impl TapRecognizer {
  pub(crate) fn update(
    &mut self,
    contact_count: usize,
    centroid: Option<(f32, f32)>,
    timestamp: f64,
  ) -> Option<TwoFingerTap> {
    if contact_count > 0 {
      let episode = self.episode.get_or_insert(Episode {
        started_at: timestamp,
        peak_touches: 0,
        origin: None,
        travel: 0.0,
      });
      episode.peak_touches = episode.peak_touches.max(contact_count);
      if contact_count == 2 {
        if let Some(point) = centroid {
          if let Some(origin) = episode.origin {
            episode.travel = episode
              .travel
              .max((point.0 - origin.0).hypot(point.1 - origin.1));
          } else {
            episode.origin = Some(point);
          }
        }
      }
      return None;
    }

    let episode = self.episode.take()?;
    (episode.peak_touches == 2
      && timestamp - episode.started_at < TAP_MAX_DURATION
      && episode.travel < TAP_MAX_TRAVEL)
      .then_some(TwoFingerTap)
  }
}

#[cfg(test)]
mod tests {
  use super::TapRecognizer;
  type Frame = (usize, Option<(f32, f32)>, f64);

  fn is_tap(frames: &[Frame]) -> bool {
    let mut recognizer = TapRecognizer::default();
    frames
      .iter()
      .filter_map(|&(count, centroid, timestamp)| recognizer.update(count, centroid, timestamp))
      .count()
      == 1
  }

  #[test]
  fn recognizes_clean_two_finger_tap() {
    assert!(is_tap(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.502, 0.501)), 0.04),
      (0, None, 0.08),
    ]));
  }

  #[test]
  fn ignores_three_finger_tap() {
    assert!(!is_tap(&[(3, Some((0.5, 0.5)), 0.0), (0, None, 0.06)]));
  }

  #[test]
  fn ignores_single_finger_tap() {
    assert!(!is_tap(&[(1, Some((0.5, 0.5)), 0.0), (0, None, 0.05)]));
  }

  #[test]
  fn ignores_slow_two_finger_press() {
    assert!(!is_tap(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.5, 0.5)), 0.2),
      (0, None, 0.4),
    ]));
  }

  #[test]
  fn ignores_short_two_finger_scroll() {
    assert!(!is_tap(&[
      (2, Some((0.5, 0.5)), 0.0),
      (2, Some((0.5, 0.6)), 0.05),
      (0, None, 0.1),
    ]));
  }

  #[test]
  fn recognizes_fingers_lifting_one_at_a_time() {
    assert!(is_tap(&[
      (2, Some((0.4, 0.4)), 0.0),
      (1, Some((0.9, 0.1)), 0.05),
      (0, None, 0.09),
    ]));
  }
}
