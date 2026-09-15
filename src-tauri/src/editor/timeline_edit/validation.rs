// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn validate(edit: &RecordingTimelineEdit) -> Result<(), String> {
  crate::editor::annotations::timing::validate_clips(&edit.annotation_clips)?;
  if edit.segments.is_empty() || edit.segments.len() > MAX_SEGMENTS {
    return Err("The timeline must contain a reasonable number of segments".to_owned());
  }
  let mut previous_end = 0.0;
  let mut ids = std::collections::HashSet::with_capacity(edit.segments.len());
  for segment in &edit.segments {
    if !segment.source_start.is_finite()
      || !segment.source_end.is_finite()
      || segment.source_start < previous_end
      || segment.source_start < 0.0
      || segment.source_end <= segment.source_start
      || segment.source_end > 1.0
      || !segment.playback_rate.is_finite()
      || !(0.25..=4.0).contains(&segment.playback_rate)
      || !ids.insert(segment.id)
    {
      return Err("The timeline contains an invalid segment".to_owned());
    }
    previous_end = segment.source_end;
  }
  if edit.next_segment_id
    <= edit
      .segments
      .iter()
      .map(|segment| segment.id)
      .max()
      .unwrap_or(0)
  {
    return Err("The timeline's next segment identity is invalid".to_owned());
  }
  keyboard::validate(&edit.keyboard_deletions, MAX_SEGMENTS)?;
  Ok(())
}
