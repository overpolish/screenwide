// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl CursorCompositor {
  pub fn open(path: &Path) -> Result<Self, String> {
    let records = cursor::read(path)?;
    Self::from_records(&records)
  }

  pub(super) fn from_records(records: &[CursorRecord]) -> Result<Self, String> {
    let source = records
      .iter()
      .find_map(|record| match record {
        CursorRecord::Header { source, .. } => Some(source.clone()),
        _ => None,
      })
      .ok_or_else(|| "The cursor recording has no source".to_owned())?;
    let mut appearances: Vec<_> = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Appearance {
          height,
          hotspot_x,
          hotspot_y,
          style,
          timestamp_us,
          width,
        } => Some(Appearance {
          height: *height,
          hotspot_x: *hotspot_x,
          hotspot_y: *hotspot_y,
          style: *style,
          timestamp_us: *timestamp_us,
          width: *width,
        }),
        _ => None,
      })
      .collect();
    appearances.sort_by_key(|appearance| appearance.timestamp_us);
    normalize_custom_fallback_size(&mut appearances);
    let visibility = visibility::events(records);
    let mut raw_positions: Vec<_> = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Visibility {
          timestamp_us, x, y, ..
        }
        | CursorRecord::Position { timestamp_us, x, y }
        | CursorRecord::Button {
          timestamp_us, x, y, ..
        } => Some(Position {
          segment: 0,
          timestamp_us: *timestamp_us,
          x: *x,
          y: *y,
        }),
        _ => None,
      })
      .collect();
    raw_positions.sort_by_key(|position| position.timestamp_us);
    // macOS's cursor smoothing is already tuned and shipped. Windows polling
    // exposes explicit held intervals that must remain exact anchors.
    let dwell_anchors = if visibility.is_empty() && cfg!(target_os = "windows") {
      dwell_anchors(&raw_positions)
    } else {
      Vec::new()
    };
    segment_raw_positions(&mut raw_positions);
    let mut positions = if visibility.is_empty() && cfg!(target_os = "windows") {
      stabilise_positions(&raw_positions, source.width)
    } else {
      // Event-driven macOS cursor positions are already the canonical path.
      // Keep its proven smoothing input free of Windows polling heuristics.
      raw_positions.clone()
    };
    visibility::segment(&mut raw_positions, &visibility);
    visibility::segment(&mut positions, &visibility);
    let recording_end_us = raw_positions
      .last()
      .map_or(0, |position| position.timestamp_us);
    let stable = stable_appearances(&appearances, recording_end_us);
    let mut raw_button_events = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Button {
          button,
          state,
          timestamp_us,
          ..
        } => Some((*timestamp_us, *button, *state)),
        _ => None,
      })
      .collect::<Vec<_>>();
    raw_button_events.sort_by_key(|(timestamp_us, ..)| *timestamp_us);
    let mut pressed = Vec::<CursorButton>::new();
    let mut button_events = Vec::new();
    for (timestamp_us, button, state) in raw_button_events {
      match state {
        ButtonState::Down if !pressed.contains(&button) => {
          if pressed.is_empty() {
            button_events.push(ButtonEvent {
              state,
              timestamp_us,
            });
          }
          pressed.push(button);
        }
        ButtonState::Up if pressed.contains(&button) => {
          pressed.retain(|pressed_button| *pressed_button != button);
          if pressed.is_empty() {
            button_events.push(ButtonEvent {
              state,
              timestamp_us,
            });
          }
        }
        _ => {}
      }
    }
    Ok(Self {
      visibility,
      appearances: stable,
      button_events,
      dwell_anchors,
      raw_positions,
      positions,
      source,
    })
  }
}
