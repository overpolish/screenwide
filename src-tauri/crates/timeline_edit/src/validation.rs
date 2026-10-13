// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn validate(edit: &RecordingTimelineEdit) -> Result<(), String> {
  screenwide_annotations::timing::validate_clips(&edit.annotation_clips)?;
  screenwide_scenes::validate_clips(&edit.scene_clips)?;
  silences::validate(&edit.silence_cuts)?;
  transcript::validate(&edit.transcript_cuts, &edit.transcript_corrections)?;
  if let Some(captions) = &edit.captions {
    captions.validate()?;
  }
  if edit.segments.is_empty() || edit.segments.len() > MAX_SEGMENTS {
    return Err("The timeline must contain a reasonable number of segments".to_owned());
  }
  let mut ids = std::collections::HashSet::with_capacity(edit.segments.len());
  for segment in &edit.segments {
    if !segment.source_start.is_finite()
      || !segment.source_end.is_finite()
      || segment.source_start < 0.0
      || segment.source_end <= segment.source_start
      || segment.source_end > 1.0
      || !segment.playback_rate.is_finite()
      || !(0.25..=4.0).contains(&segment.playback_rate)
      || !ids.insert(segment.id)
    {
      return Err("The timeline contains an invalid segment".to_owned());
    }
  }
  // Segments play in list order, which a reordered timeline leaves out of
  // source order; each moment of the recording still plays at most once.
  let mut spans: Vec<(f64, f64)> = edit
    .segments
    .iter()
    .map(|segment| (segment.source_start, segment.source_end))
    .collect();
  spans.sort_by(|a, b| a.0.total_cmp(&b.0));
  if spans.windows(2).any(|pair| pair[1].0 < pair[0].1) {
    return Err("The timeline plays part of the recording twice".to_owned());
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
