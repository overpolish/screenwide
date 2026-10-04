// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How often a GPU export says how far it has got.
//!
//! Both export loops can report after every frame, a hundred or more times a
//! second. Each report reaches the editor's webview as an event, and each
//! event re-rendered and repainted the window, which kept a CPU core busy and
//! had the system compositor redraw the screen on the GPU the export runs on.
//! Ten reports a second still move the bar smoothly.

use std::time::{Duration, Instant};

/// The shortest time between two progress reports.
const INTERVAL: Duration = Duration::from_millis(100);

#[derive(Default)]
pub(super) struct ProgressPace {
  last: Option<Instant>,
}

impl ProgressPace {
  /// Whether a report made `now` goes out: the first, the one that
  /// `finishes` the pass, and otherwise one an interval after the last.
  pub(super) fn due(&mut self, now: Instant, finishes: bool) -> bool {
    let due = finishes
      || self
        .last
        .is_none_or(|last| now.saturating_duration_since(last) >= INTERVAL);
    if due {
      self.last = Some(now);
    }
    due
  }
}
