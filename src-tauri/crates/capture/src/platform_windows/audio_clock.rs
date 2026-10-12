// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl AudioOnlyClock {
  pub(super) fn new(started: Instant) -> Self {
    Self {
      paused: Mutex::new((None, Duration::ZERO)),
      started,
    }
  }

  pub(super) fn pause(&self, at: Instant) {
    if let Ok(mut state) = self.paused.lock() {
      state.0.get_or_insert(at);
    }
  }

  pub(super) fn resume(&self, at: Instant) {
    if let Ok(mut state) = self.paused.lock() {
      if let Some(started) = state.0.take() {
        state.1 = state
          .1
          .saturating_add(at.saturating_duration_since(started));
      }
    }
  }

  pub(super) fn duration_ms(&self, at: Instant) -> u64 {
    let elapsed = at.saturating_duration_since(self.started);
    let paused = self
      .paused
      .lock()
      .map(|state| {
        state.1.saturating_add(
          state
            .0
            .map_or(Duration::ZERO, |pause| at.saturating_duration_since(pause)),
        )
      })
      .unwrap_or(Duration::ZERO);
    u64::try_from(elapsed.saturating_sub(paused).as_millis()).unwrap_or(u64::MAX)
  }
}
