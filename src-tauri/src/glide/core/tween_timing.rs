// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! TEMPORARY(#22): per-tween frame timing for the laggy-resize
//! investigation: how long the window's application holds each write, and
//! how far apart the writes really land. Each line is appended to
//! `screenwide-glide-tween.log` in the temporary directory, so the timing is
//! kept however the app was started.

use std::time::Instant;

pub(super) struct Timing {
  /// The window's application, where the platform can name it.
  app: Option<String>,
  frames: u32,
  last_written: Option<Instant>,
  place_total_ms: f64,
  place_max_ms: f64,
  gap_max_ms: f64,
}

impl Timing {
  pub(super) fn new(app: Option<String>) -> Self {
    Self {
      app,
      frames: 0,
      last_written: None,
      place_total_ms: 0.0,
      place_max_ms: 0.0,
      gap_max_ms: 0.0,
    }
  }

  /// Times one write, `place`, and logs the line `describe` makes of the
  /// frame's number, the write's milliseconds, the gap since the last write
  /// and whether the window took it.
  pub(super) fn frame(
    &mut self,
    place: impl FnOnce() -> bool,
    describe: impl FnOnce(u32, f64, f64, bool) -> String,
  ) -> bool {
    let began = Instant::now();
    let gap_ms = self
      .last_written
      .map_or(0.0, |last| (began - last).as_secs_f64() * 1000.0);
    let placed = place();
    let place_ms = began.elapsed().as_secs_f64() * 1000.0;
    self.frames += 1;
    self.last_written = Some(began);
    self.place_total_ms += place_ms;
    self.place_max_ms = self.place_max_ms.max(place_ms);
    self.gap_max_ms = self.gap_max_ms.max(gap_ms);
    log(&describe(self.frames, place_ms, gap_ms, placed));
    placed
  }

  /// Times the settling step, `settle`.
  pub(super) fn settle(settle: impl FnOnce()) {
    let began = Instant::now();
    settle();
    log(&format!(
      "settle {:.1}ms",
      began.elapsed().as_secs_f64() * 1000.0
    ));
  }

  /// Logs the tween's summary once it has settled.
  pub(super) fn done(&self, resizes: bool, took_ms: f64, planned_ms: f64) {
    log(&format!(
      "done app={} resizes={resizes} frames={} took={took_ms:.0}ms (planned {planned_ms:.0}ms) \
       place avg={:.1}ms max={:.1}ms gap max={:.1}ms",
      self.app.as_deref().unwrap_or("-"),
      self.frames,
      self.place_total_ms / f64::from(self.frames.max(1)),
      self.place_max_ms,
      self.gap_max_ms
    ));
  }
}

fn log(line: &str) {
  use std::io::Write;
  let path = std::env::temp_dir().join("screenwide-glide-tween.log");
  if let Ok(mut file) = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
  {
    let _ = writeln!(file, "{line}");
  }
}
