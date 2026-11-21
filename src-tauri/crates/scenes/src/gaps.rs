// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A gap, time inserted into the timeline with nothing recorded behind it,
//! is drawn as a blank scene of its own: the background, its own where it
//! has one, and what is drawn over it, arriving and leaving as any scene
//! does. No scene of the recording plays over a gap; one that would is cut
//! back to either side of it.

use screenwide_background::Background;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::model::{RecordingSceneClip, RecordingScenePreset};

/// A gap on the timeline, in milliseconds of the edited video, and the
/// background it shows where it has one of its own.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GapScene {
  pub start_ms: u64,
  pub end_ms: u64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub background: Option<Background>,
}

impl GapScene {
  /// The blank scene the gap is drawn as.
  fn scene(&self) -> RecordingSceneClip {
    RecordingSceneClip {
      id: format!("gap-{}", self.start_ms),
      start_ms: self.start_ms,
      end_ms: self.end_ms,
      preset: RecordingScenePreset::Blank,
      screen: None,
      camera: None,
      follow_face: None,
      cursor: None,
      background: self.background.clone(),
      boxes: None,
      radius: None,
      inset: None,
      rotation: None,
      variant: None,
      layout_before: None,
      shapes: None,
      auto: false,
    }
  }
}

/// `clips` as the picture plays them with `gaps`: each gap a blank scene of
/// its own, and every clip cut back to either side of the gaps it crosses,
/// in order.
pub fn with_gaps(clips: &[RecordingSceneClip], gaps: &[GapScene]) -> Vec<RecordingSceneClip> {
  let mut staged: Vec<RecordingSceneClip> = Vec::with_capacity(clips.len() + gaps.len() * 2);
  for clip in clips {
    let mut start = clip.start_ms;
    for gap in gaps.iter().filter(|gap| gap.start_ms < clip.end_ms) {
      if gap.end_ms <= start {
        continue;
      }
      if gap.start_ms > start {
        staged.push(RecordingSceneClip {
          start_ms: start,
          end_ms: gap.start_ms,
          ..clip.clone()
        });
      }
      start = start.max(gap.end_ms);
    }
    if start < clip.end_ms {
      staged.push(RecordingSceneClip {
        start_ms: start,
        ..clip.clone()
      });
    }
  }
  staged.extend(gaps.iter().map(GapScene::scene));
  staged.sort_by_key(|clip| clip.start_ms);
  staged
}

#[cfg(test)]
mod tests {
  use super::*;

  fn scene(id: &str, [start_ms, end_ms]: [u64; 2]) -> RecordingSceneClip {
    serde_json::from_value(serde_json::json!({
      "id": id, "startMs": start_ms, "endMs": end_ms, "preset": "picture-in-picture"
    }))
    .unwrap()
  }

  #[test]
  fn a_gap_is_a_blank_scene_and_no_scene_plays_over_it() {
    let gaps = [GapScene {
      start_ms: 4_000,
      end_ms: 6_000,
      background: None,
    }];
    let staged = with_gaps(
      &[scene("a", [1_000, 3_000]), scene("b", [3_500, 8_000])],
      &gaps,
    );
    let spans: Vec<_> = staged
      .iter()
      .map(|clip| (clip.id.as_str(), clip.start_ms, clip.end_ms, clip.preset))
      .collect();
    assert_eq!(
      spans,
      vec![
        ("a", 1_000, 3_000, RecordingScenePreset::PictureInPicture),
        ("b", 3_500, 4_000, RecordingScenePreset::PictureInPicture),
        ("gap-4000", 4_000, 6_000, RecordingScenePreset::Blank),
        ("b", 6_000, 8_000, RecordingScenePreset::PictureInPicture),
      ]
    );
  }
}
