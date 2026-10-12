// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl StreamWriter {
  pub(super) fn write(&mut self, record: &CursorRecord) -> Result<(), String> {
    self.output.write(record)
  }

  pub(super) fn record(&mut self, event: RawCursorEvent) -> Result<bool, String> {
    let timestamp_us = if matches!(event.kind, RawCursorEventKind::Snapshot) {
      self.clock.initial_timestamp_us()
    } else {
      self.clock.timestamp_us(event.at)
    };
    let Some(timestamp_us) = timestamp_us else {
      return Ok(false);
    };
    if matches!(event.kind, RawCursorEventKind::AppSwitch) {
      self.write(&CursorRecord::AppSwitch { timestamp_us })?;
      self.output.flush()?;
      self.last_flush = event.at;
      return Ok(true);
    }

    if self.last_visibility == Some(false)
      && matches!(
        event.kind,
        RawCursorEventKind::Move | RawCursorEventKind::Appearance
      )
    {
      return Ok(true);
    }
    self.last_position = Some((event.x, event.y));
    if self.last_appearance.as_ref() != Some(&event.appearance) {
      self.write(&CursorRecord::Appearance {
        height: event.appearance.height,
        hotspot_x: event.appearance.hotspot_x,
        hotspot_y: event.appearance.hotspot_y,
        style: event.appearance.style,
        timestamp_us,
        width: event.appearance.width,
      })?;
      self.last_appearance = Some(event.appearance.clone());
    }

    match event.kind {
      RawCursorEventKind::Appearance | RawCursorEventKind::AppSwitch => {}
      RawCursorEventKind::Move | RawCursorEventKind::Snapshot => {
        if matches!(event.kind, RawCursorEventKind::Move)
          && self
            .last_move
            .is_some_and(|last| event.at.saturating_duration_since(last) < MOVEMENT_INTERVAL)
        {
          return Ok(true);
        }
        self.write(&CursorRecord::Position {
          timestamp_us,
          x: event.x,
          y: event.y,
        })?;
        self.last_move = Some(event.at);
      }
      RawCursorEventKind::Button {
        button,
        click_count,
        state,
      } => {
        self.write(&CursorRecord::Button {
          button,
          click_count,
          state,
          timestamp_us,
          x: event.x,
          y: event.y,
        })?;
        self.output.flush()?;
        self.last_flush = event.at;
      }
    }

    if event.at.saturating_duration_since(self.last_flush) >= FLUSH_INTERVAL {
      self.output.flush()?;
      self.last_flush = event.at;
    }
    Ok(true)
  }
}
