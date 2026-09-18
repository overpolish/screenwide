// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::sync::Weak;

pub(super) struct VisibilityState {
  pub visible: bool,
  pub streams: Vec<Weak<Mutex<StreamWriter>>>,
}

pub(super) static VISIBILITY: Mutex<VisibilityState> = Mutex::new(VisibilityState {
  visible: true,
  streams: Vec::new(),
});

/// Records that the pointer has actually gone or come back, including a warp
/// that emits no mouse event of its own.
///
/// Glide hides it for the length of a gesture; live annotation hides it for as
/// long as the overlay is up, where the pointer is a crosshair drawing an
/// annotation rather than anything a viewer should follow. The two cannot
/// overlap: an active overlay blocks Glide.
pub(crate) fn set_cursor_visibility(visible: bool, position: Option<(f64, f64)>) {
  let mut visibility = VISIBILITY.lock().unwrap_or_else(|p| p.into_inner());
  visibility.visible = visible;
  let at = Instant::now();
  visibility.streams.retain(|stream| {
    let Some(stream) = stream.upgrade() else {
      return false;
    };
    let mut stream = stream.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(timestamp_us) = stream.clock.timestamp_us(at) {
      if let Err(error) = stream.record_visibility(timestamp_us, visible, position) {
        stream.failure = Some(error);
      }
    }
    true
  });
}

impl StreamWriter {
  pub(super) fn record_visibility(
    &mut self,
    timestamp_us: u64,
    visible: bool,
    position: Option<(f64, f64)>,
  ) -> Result<(), String> {
    if self.last_visibility == Some(visible) {
      return Ok(());
    }
    let (x, y) = position.or(self.last_position).unwrap_or((0.0, 0.0));
    // Ordinary recordings keep the same initial records as version 1.
    if !visible || self.last_visibility == Some(false) {
      self.write(&CursorRecord::Visibility {
        timestamp_us,
        visible,
        x,
        y,
      })?;
      self.writer.flush().map_err(|error| error.to_string())?;
    }
    self.last_visibility = Some(visible);
    self.last_position = Some((x, y));
    self.last_move = None;
    Ok(())
  }
}

pub(super) fn sink(state: &Arc<Mutex<StreamWriter>>) -> EventSink {
  VISIBILITY
    .lock()
    .unwrap_or_else(|p| p.into_inner())
    .streams
    .push(Arc::downgrade(state));
  let state = Arc::clone(state);
  Arc::new(move |event| {
    let visibility = VISIBILITY.lock().unwrap_or_else(|p| p.into_inner());
    let mut state = state.lock().unwrap_or_else(|p| p.into_inner());
    if state.failure.is_some() {
      return false;
    }
    let timestamp = if matches!(event.kind, RawCursorEventKind::Snapshot) {
      state.clock.initial_timestamp_us()
    } else {
      state.clock.timestamp_us(event.at)
    };
    let result = timestamp
      .map_or(Ok(()), |timestamp| {
        state.record_visibility(timestamp, visibility.visible, Some((event.x, event.y)))
      })
      .and_then(|()| state.record(event));
    match result {
      Ok(recording) => recording,
      Err(error) => {
        eprintln!("Cursor recording stopped writing: {error}");
        state.failure = Some(error);
        false
      }
    }
  })
}
