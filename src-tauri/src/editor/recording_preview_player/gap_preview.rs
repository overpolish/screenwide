// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The timeline's gaps in the preview: moments past the recording's end,
//! where nothing is decoded and a blank scene of the gap's own is drawn, as
//! the export draws it.

use screenwide_scenes::{with_gaps, GapScene, RecordingSceneClip};
use screenwide_timeline::TimelineRange;

use super::scene_preview::PreviewScenes;
use super::PlayerSources;

/// The recording's own scene clips, which gestures and annotations read, and
/// the clips the picture is drawn by: those with the gaps among them as
/// blank scenes ([`with_gaps`]), kept beside them so no frame makes them.
#[derive(Default)]
pub(super) struct SceneClips {
  own: Vec<RecordingSceneClip>,
  gaps: Vec<GapScene>,
  drawn: Vec<RecordingSceneClip>,
}

impl SceneClips {
  pub(super) fn new(own: Vec<RecordingSceneClip>, gaps: Vec<GapScene>) -> Self {
    let drawn = with_gaps(&own, &gaps);
    Self { own, gaps, drawn }
  }

  pub(super) fn own(&self) -> &[RecordingSceneClip] {
    &self.own
  }

  pub(super) fn gaps(&self) -> &[GapScene] {
    &self.gaps
  }

  pub(super) fn drawn(&self) -> &[RecordingSceneClip] {
    &self.drawn
  }

  /// Takes `own` and, where given, `gaps`, answering whether either changed.
  pub(super) fn set(&mut self, own: Vec<RecordingSceneClip>, gaps: Option<Vec<GapScene>>) -> bool {
    let changed = own != self.own || gaps.as_ref().is_some_and(|gaps| *gaps != self.gaps);
    let gaps = gaps.unwrap_or_else(|| std::mem::take(&mut self.gaps));
    *self = Self::new(own, gaps);
    changed
  }
}

impl PreviewScenes {
  /// Whether the moment `source_ms` lies in a gap: past the recording's end,
  /// in a stretch the timeline plays.
  pub(super) fn is_gap(&self, source_ms: u64) -> bool {
    let source_us = source_ms.saturating_mul(1_000);
    source_us >= self.source_duration_us
      && self
        .ranges
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .any(|range| range.source_start_us <= source_us && source_us < range.source_end_us)
  }

  /// The recorded moment a still in a gap at `source_ms` decodes, for the
  /// panes fading under it: where the recording was left before the gap,
  /// past any gaps meeting it, as playback holds the frame in hand; or,
  /// for a gap the video starts with, where it picks up after. `None`
  /// outside a gap.
  pub(super) fn held_ms(&self, source_ms: u64) -> Option<u64> {
    let source_us = source_ms.saturating_mul(1_000);
    if source_us < self.source_duration_us {
      return None;
    }
    let ranges = self
      .ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let index = ranges
      .iter()
      .position(|range| range.source_start_us <= source_us && source_us < range.source_end_us)?;
    let recorded = |range: &&TimelineRange| !range.is_gap(self.source_duration_us);
    ranges[..index]
      .iter()
      .rev()
      .find(recorded)
      .map(|before| before.source_end_us.saturating_sub(1_000) / 1_000)
      .or_else(|| {
        ranges[index + 1..]
          .iter()
          .find(recorded)
          .map(|after| after.source_start_us / 1_000)
      })
  }
}

impl PlayerSources {
  /// The moment a still at `position_ms` shows: a gap's own, which lies past
  /// the recording's end, or else the last the recording holds at most.
  pub(super) fn moment_ms(&self, position_ms: u64) -> u64 {
    if self.scenes.is_gap(position_ms) {
      position_ms
    } else {
      position_ms.min(self.duration_ms.saturating_sub(1))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A 10 s recording's scenes, played as `ranges` lay it out, each
  /// `[output_start, source_start, source_end]` in seconds.
  fn scenes(ranges: &[[u64; 3]]) -> PreviewScenes {
    let ranges = ranges
      .iter()
      .map(|&[output, start, end]| TimelineRange {
        output_start_us: output * 1_000_000,
        source_start_us: start * 1_000_000,
        source_end_us: end * 1_000_000,
        playback_rate: 1.0,
      })
      .collect();
    PreviewScenes::new((Vec::new(), Vec::new()), ranges, 10_000, None)
  }

  #[test]
  fn a_gap_holds_the_frame_before_it_or_after_it_at_the_start() {
    // 0 to 4 s, a 2 s gap, then 4 to 10 s.
    let parted = scenes(&[[0, 0, 4], [4, 20, 22], [6, 4, 10]]);
    assert_eq!(parted.held_ms(21_000), Some(3_999));
    assert_eq!(parted.held_ms(5_000), None);
    // A gap the video starts with shows what comes after it.
    let opening = scenes(&[[0, 20, 22], [2, 1, 10]]);
    assert_eq!(opening.held_ms(20_500), Some(1_000));
  }
}
