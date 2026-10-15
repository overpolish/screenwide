// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stretches of the timeline where the camera's background is taken
//! away, and what shows behind the person in each. Outside every range the
//! camera is drawn as recorded.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What shows behind the person where a range takes the background away.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CameraRangeBackground {
  /// Nothing: the person alone, over whatever the canvas has behind.
  Transparent,
  /// The room, blurred.
  Blurred,
  /// The range's own picture.
  Image,
}

/// One stretch of the timeline, `[start_ms, end_ms)`, where the camera's
/// background is taken away.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraBackgroundRange {
  /// Names the range's picture file in the project, so it is kept to
  /// letters, digits, `-` and `_`.
  pub id: String,
  pub start_ms: u64,
  pub end_ms: u64,
  pub background: CameraRangeBackground,
}

/// The longest id a range may have.
const MAX_ID_LENGTH: usize = 64;

/// Whether `ranges` can be drawn: each spans time and has an id of its own
/// that is safe in a file name, and they run in order without overlapping,
/// since the camera has one background at a time. Bounds are half open, so
/// one range may end exactly where the next begins.
pub(crate) fn validate(ranges: &[CameraBackgroundRange]) -> Result<(), String> {
  let mut ids = std::collections::HashSet::with_capacity(ranges.len());
  let mut previous_end = 0;
  for range in ranges {
    let safe_id = !range.id.is_empty()
      && range.id.len() <= MAX_ID_LENGTH
      && range
        .id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !safe_id
      || !ids.insert(range.id.as_str())
      || range.end_ms <= range.start_ms
      || range.start_ms < previous_end
    {
      return Err("The camera's background ranges are invalid".to_owned());
    }
    previous_end = range.end_ms;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn range(id: &str, start_ms: u64, end_ms: u64) -> CameraBackgroundRange {
    CameraBackgroundRange {
      id: id.to_owned(),
      start_ms,
      end_ms,
      background: CameraRangeBackground::Blurred,
    }
  }

  #[test]
  fn ranges_that_butt_are_valid() {
    assert!(validate(&[range("a", 0, 1_000), range("b", 1_000, 2_000)]).is_ok());
  }

  #[test]
  fn overlapping_ranges_are_rejected() {
    assert!(validate(&[range("a", 0, 1_500), range("b", 1_000, 2_000)]).is_err());
  }

  #[test]
  fn an_id_that_could_leave_the_project_folder_is_rejected() {
    assert!(validate(&[range("../picture", 0, 1_000)]).is_err());
    assert!(validate(&[range("", 0, 1_000)]).is_err());
  }
}
