// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! TEMPORARY(#12): timing for each editor layout that changes the viewport's
//! size, appended to `screenwide-editor-resize.log` in the temporary
//! directory so a release build keeps it however the app was started. It
//! separates the frontend's cadence (`gap`) from the time the layout command
//! spends, splits that time into the command's phases and, on Windows, counts
//! its draws and compositor waits. The frontend's own share before the
//! command (measuring the page, waiting behind the layout in flight, the
//! round trip of the one before) rides in with the layout.

use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
struct State {
  size: Option<(i64, i64)>,
  last_at: Option<Instant>,
  /// Whether a size-changing layout is under way, so only its draws,
  /// commits and phases are counted.
  measuring: bool,
  draw: Spent,
  commit: Spent,
  phase_at: Option<Instant>,
  phases: Vec<(&'static str, Duration)>,
}

#[derive(Default)]
struct Spent {
  count: u32,
  total: Duration,
  max: Duration,
}

impl Spent {
  #[cfg(windows)]
  fn add(&mut self, took: Duration) {
    self.count += 1;
    self.total += took;
    self.max = self.max.max(took);
  }

  fn describe(&self) -> String {
    if self.count == 0 {
      return "-".to_owned();
    }
    format!(
      "{:.1}ms (n={} max={:.1}ms)",
      ms(self.total),
      self.count,
      ms(self.max)
    )
  }
}

/// What the frontend spent on a layout before sending it, in milliseconds.
#[derive(serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrontendTiming {
  /// Measuring the page and building the layout.
  measure_ms: Option<f64>,
  /// From the layout being ready to it being sent: the wait behind the
  /// layout in flight.
  queued_ms: f64,
  /// From the previous layout's answer to this one being sent.
  idle_ms: Option<f64>,
  /// The previous layout's invoke, from sending it to its answer.
  previous_round_trip_ms: Option<f64>,
  /// The layout's size as JSON, in characters.
  payload_chars: u64,
}

impl FrontendTiming {
  fn describe(&self) -> String {
    let optional =
      |value: Option<f64>| value.map_or_else(|| "-".to_owned(), |value| format!("{value:.1}ms"));
    format!(
      "measure={} queued={:.1}ms idle={} prev-roundtrip={} payload={}",
      optional(self.measure_ms),
      self.queued_ms,
      optional(self.idle_ms),
      optional(self.previous_round_trip_ms),
      self.payload_chars
    )
  }
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Writes its layout's line when dropped, at the end of the command.
pub(crate) struct Layout {
  kind: &'static str,
  size: (i64, i64),
  gap: Option<Duration>,
  began: Instant,
  frontend: Option<FrontendTiming>,
}

/// Starts timing a layout command whose viewport is `width` by `height`
/// logical points. Answers `None` when the size is the one last laid out.
pub(crate) fn begin(
  kind: &'static str,
  width: f64,
  height: f64,
  frontend: Option<FrontendTiming>,
) -> Option<Layout> {
  let size = (width.round() as i64, height.round() as i64);
  let began = Instant::now();
  let mut slot = STATE.lock().ok()?;
  let state = slot.get_or_insert_with(State::default);
  if state.size == Some(size) {
    return None;
  }
  let gap = state.last_at.map(|last| began - last);
  state.size = Some(size);
  state.last_at = Some(began);
  state.measuring = true;
  state.draw = Spent::default();
  state.commit = Spent::default();
  state.phase_at = Some(began);
  state.phases.clear();
  Some(Layout {
    kind,
    size,
    gap,
    began,
    frontend,
  })
}

/// Ends the command's phase named `name` here; the next one starts.
pub(crate) fn mark(name: &'static str) {
  let now = Instant::now();
  if let Ok(mut slot) = STATE.lock() {
    if let Some(state) = slot.as_mut().filter(|state| state.measuring) {
      if let Some(at) = state.phase_at.replace(now) {
        state.phases.push((name, now - at));
      }
    }
  }
}

/// One workspace draw: the swap chain matched to the viewport, the scene
/// drawn into it and presented. Only Windows's surface reports these.
#[cfg(windows)]
pub(crate) fn record_draw(took: Duration) {
  record(took, |state| &mut state.draw);
}

/// One compositor commit, with its wait when it waited.
#[cfg(windows)]
pub(crate) fn record_commit(took: Duration) {
  record(took, |state| &mut state.commit);
}

#[cfg(windows)]
fn record(took: Duration, spent: impl FnOnce(&mut State) -> &mut Spent) {
  if let Ok(mut slot) = STATE.lock() {
    if let Some(state) = slot.as_mut().filter(|state| state.measuring) {
      spent(state).add(took);
    }
  }
}

impl Drop for Layout {
  fn drop(&mut self) {
    mark("rest");
    let command = self.began.elapsed();
    let Some((draw, commit, phases)) = STATE.lock().ok().and_then(|mut slot| {
      let state = slot.as_mut()?;
      state.measuring = false;
      let phases = state
        .phases
        .iter()
        .map(|(name, took)| format!("{name}:{:.1}", ms(*took)))
        .collect::<Vec<_>>()
        .join(" ");
      Some((state.draw.describe(), state.commit.describe(), phases))
    }) else {
      return;
    };
    let gap = self
      .gap
      .map_or_else(|| "-".to_owned(), |gap| format!("{:.1}ms", ms(gap)));
    let frontend = self
      .frontend
      .map_or_else(|| "-".to_owned(), |frontend| frontend.describe());
    write(format!(
      "{} size={}x{} gap={gap} command={:.1}ms draw={draw} commit={commit} phases=[{phases}] \
       frontend=[{frontend}]",
      self.kind,
      self.size.0,
      self.size.1,
      ms(command)
    ));
  }
}

fn ms(duration: Duration) -> f64 {
  duration.as_secs_f64() * 1000.0
}

fn write(line: String) {
  use std::io::Write;
  let path = std::env::temp_dir().join("screenwide-editor-resize.log");
  if let Ok(mut file) = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
  {
    let _ = writeln!(file, "{line}");
  }
}
