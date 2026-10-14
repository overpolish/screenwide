// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a caption moves as it comes on and goes off, as data: for the
//! caption coming on whole, for each word coming on as it is said, and for
//! the caption and its words going off once it has been read, tracks of
//! keyframes over the properties in [`property`]. A track holds a property
//! at its first keyframe's value until then and at its last one's after,
//! easing into each keyframe from the one before. A motion can also be
//! written as a preset's name, which is how the templates and older saved
//! styles name the ones the app ships.

mod pose;
mod presets;
mod property;
#[cfg(test)]
mod tests;

use serde::{de, Deserialize, Deserializer, Serialize};
use ts_rs::TS;

pub(crate) use pose::{pose, Mirror, Pose};
pub use property::CaptionProperty;

/// The longest a motion runs, so a template cannot leave a caption still
/// moving long after it has been read.
pub const MAX_MOTION_MS: f64 = 2_000.0;
/// The most keyframes one track holds.
pub const MAX_KEYFRAMES: usize = 8;
/// The furthest a back ease overshoots.
const MAX_OVERSHOOT: f64 = 5.0;

/// How a keyframe is eased into from the one before it.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionEasing {
  #[default]
  Linear,
  EaseIn,
  EaseOut,
  EaseInOut,
  /// Past the keyframe's value and back to it, `overshoot` setting how far.
  BackOut {
    overshoot: f64,
  },
}

impl CaptionEasing {
  fn is_linear(&self) -> bool {
    *self == Self::Linear
  }

  /// How far along `progress`, from 0 to 1, has come once eased.
  pub(crate) fn eased(self, progress: f64) -> f64 {
    match self {
      Self::Linear => progress,
      Self::EaseIn => progress * progress,
      Self::EaseOut => 1.0 - (1.0 - progress).powi(2),
      Self::EaseInOut if progress < 0.5 => 2.0 * progress * progress,
      Self::EaseInOut => 1.0 - 2.0 * (1.0 - progress).powi(2),
      Self::BackOut { overshoot } => {
        let from_end = progress - 1.0;
        1.0 + (overshoot + 1.0) * from_end.powi(3) + overshoot * from_end.powi(2)
      }
    }
  }

  fn normalized(self) -> Self {
    match self {
      Self::BackOut { overshoot } => Self::BackOut {
        overshoot: in_range(overshoot, (0.0, MAX_OVERSHOOT), 0.0),
      },
      other => other,
    }
  }
}

/// A property's value `at_ms` after its motion starts, eased into from the
/// keyframe before; linear where no easing is written.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionKeyframe {
  pub at_ms: f64,
  pub value: f64,
  #[serde(default, skip_serializing_if = "CaptionEasing::is_linear")]
  #[ts(optional, as = "Option<CaptionEasing>")]
  pub easing: CaptionEasing,
}

/// One property's keyframes, in time order. An offset track `from_edge`
/// measures its offsets from the edge of the picture the caption sits
/// against rather than in the picture's own directions: a caption along the
/// top moves up where one along the bottom moves down, and one at the right
/// moves right where one at the left moves left, so a word coming in from
/// its edge does so wherever the caption is placed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionMotionTrack {
  pub property: CaptionProperty,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(optional, as = "Option<bool>")]
  pub from_edge: bool,
  pub keyframes: Vec<CaptionKeyframe>,
}

/// How a caption comes on and goes off: `caption` for the caption coming
/// on whole, `word` for each word coming on as it is said, where words are
/// revealed, and `caption_out` and `word_out` for the caption and each of
/// its words going off. An exit is timed to finish as its caption's time
/// ends, its keyframes timed from when it starts, so a caption is gone by
/// the edge of its clip, as everything else on the timeline is. A trigger
/// left out does not move. Each field defaults on its own: the whole
/// motion's default is a preset, read through this very type.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionMotion {
  #[serde(default)]
  pub caption: Vec<CaptionMotionTrack>,
  #[serde(default)]
  pub word: Vec<CaptionMotionTrack>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<CaptionMotionTrack>>")]
  pub caption_out: Vec<CaptionMotionTrack>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<CaptionMotionTrack>>")]
  pub word_out: Vec<CaptionMotionTrack>,
}

impl Default for CaptionMotion {
  /// No motion for the caption, and words fading in as they are said.
  fn default() -> Self {
    presets::named("fade").expect("the bundled motion presets have a fade")
  }
}

/// Written whole, or as a preset's name.
pub(crate) fn deserialize_motion<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<CaptionMotion, D::Error> {
  match serde_json::Value::deserialize(deserializer)? {
    serde_json::Value::String(name) => presets::named(&name)
      .ok_or_else(|| de::Error::custom(format!("there is no motion preset called `{name}`"))),
    whole => serde_json::from_value(whole).map_err(de::Error::custom),
  }
}

/// `value` within `range`, or `fallback` where it is no number at all.
fn in_range(value: f64, (min, max): (f64, f64), fallback: f64) -> f64 {
  if value.is_finite() {
    value.clamp(min, max)
  } else {
    fallback
  }
}

fn normalized_tracks(tracks: &[CaptionMotionTrack]) -> Vec<CaptionMotionTrack> {
  let mut kept: Vec<CaptionMotionTrack> = Vec::new();
  // One track a property: a second would only fight the first.
  for track in tracks {
    if kept.iter().any(|other| other.property == track.property) {
      continue;
    }
    let spec = track.property.spec();
    let mut keyframes: Vec<CaptionKeyframe> = track
      .keyframes
      .iter()
      .take(MAX_KEYFRAMES)
      .map(|keyframe| CaptionKeyframe {
        at_ms: in_range(keyframe.at_ms, (0.0, MAX_MOTION_MS), 0.0),
        value: in_range(keyframe.value, spec.range, spec.rest),
        easing: keyframe.easing.normalized(),
      })
      .collect();
    keyframes.sort_by(|a, b| a.at_ms.total_cmp(&b.at_ms));
    kept.push(CaptionMotionTrack {
      property: track.property,
      from_edge: track.from_edge,
      keyframes,
    });
  }
  kept
}

impl CaptionMotion {
  /// The motion with every track and keyframe within its limits.
  pub(crate) fn normalized(&self) -> Self {
    Self {
      caption: normalized_tracks(&self.caption),
      word: normalized_tracks(&self.word),
      caption_out: normalized_tracks(&self.caption_out),
      word_out: normalized_tracks(&self.word_out),
    }
  }

  /// How long after it starts every entrance track has reached its last
  /// keyframe.
  pub(crate) fn settles_ms(&self) -> f64 {
    last_keyframe_ms(self.caption.iter().chain(&self.word))
  }

  /// How long a caption's exit runs before the end of its time: until
  /// every exit track has reached its last keyframe.
  pub(crate) fn exit_ms(&self) -> f64 {
    last_keyframe_ms(self.caption_out.iter().chain(&self.word_out))
  }

  /// How long a word takes to come on, which a rolling caption's words
  /// slide along in as it does.
  pub(crate) fn word_entrance_ms(&self) -> f64 {
    last_keyframe_ms(self.word.iter())
  }

  /// How long a word takes to go off.
  pub(crate) fn word_exit_ms(&self) -> f64 {
    last_keyframe_ms(self.word_out.iter())
  }

  /// Whether it moves anything as a caption goes off, which only a newer
  /// manifest format can say.
  pub(crate) fn has_exit(&self) -> bool {
    !self.caption_out.is_empty() || !self.word_out.is_empty()
  }

  /// Whether it turns anything, which only a newer manifest format can say.
  pub(crate) fn turns(&self) -> bool {
    [&self.caption, &self.word, &self.caption_out, &self.word_out]
      .into_iter()
      .flatten()
      .any(|track| {
        matches!(
          track.property,
          CaptionProperty::Rotate | CaptionProperty::RotateX | CaptionProperty::RotateY
        )
      })
  }
}

fn last_keyframe_ms<'a>(tracks: impl Iterator<Item = &'a CaptionMotionTrack>) -> f64 {
  tracks
    .filter_map(|track| track.keyframes.last())
    .map(|keyframe| keyframe.at_ms)
    .fold(0.0, f64::max)
}
