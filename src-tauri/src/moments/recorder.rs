// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use super::format::{header, placed_at_us, MomentRecord};
use super::settings::MomentKind;
use crate::recording::clock::SidecarClock;

/// The recording a pressed kind shortcut lands in. The shortcut's native
/// callback has no route to the recording's handles, which the state machine
/// keeps behind locks of its own, so the running recorder is published here.
static ACTIVE: Mutex<Option<Arc<MomentRecorder>>> = Mutex::new(None);

fn active() -> MutexGuard<'static, Option<Arc<MomentRecorder>>> {
  ACTIVE
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) struct MomentRecorder {
  path: PathBuf,
  stream: Mutex<Stream>,
}

struct Stream {
  clock: SidecarClock,
  count: usize,
  failure: Option<String>,
  writer: BufWriter<File>,
}

impl MomentRecorder {
  /// Opens the sidecar at `path` on the recording clock `origin`, and makes
  /// it the one kind shortcuts write to.
  pub(crate) fn start(path: PathBuf, origin: Arc<OnceLock<Instant>>) -> Result<Arc<Self>, String> {
    let recorder = Arc::new(Self::open(path, origin)?);
    *active() = Some(Arc::clone(&recorder));
    Ok(recorder)
  }

  fn open(path: PathBuf, origin: Arc<OnceLock<Instant>>) -> Result<Self, String> {
    let file = File::create(&path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(file);
    write_line(&mut writer, &header())?;
    Ok(Self {
      path,
      stream: Mutex::new(Stream {
        clock: SidecarClock::new(origin),
        count: 0,
        failure: None,
        writer,
      }),
    })
  }

  fn stream(&self) -> MutexGuard<'_, Stream> {
    self
      .stream
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  pub(crate) fn pause(&self, at: Instant) {
    self.stream().clock.pause(at);
  }

  pub(crate) fn resume(&self, at: Instant) {
    self.stream().clock.resume(at);
  }

  /// Writes a moment of `kind` for a press at `at`. Nothing is written
  /// before the first frame or while paused, since neither has a place on
  /// the recording's timeline.
  fn record(&self, kind: &MomentKind, at: Instant) -> bool {
    let mut stream = self.stream();
    if stream.failure.is_some() {
      return false;
    }
    let Some(pressed_us) = stream.clock.timestamp_us(at) else {
      return false;
    };
    let record = MomentRecord::Moment {
      color: kind.color.clone(),
      kind_id: kind.id.clone(),
      name: kind.name.clone(),
      timestamp_us: placed_at_us(pressed_us),
    };
    // Flushed with every moment: there are few, and each has to survive the
    // app dying before the recording stops.
    match write_line(&mut stream.writer, &record) {
      Ok(()) => {
        stream.count += 1;
        true
      }
      Err(error) => {
        eprintln!("Moments stopped writing: {error}");
        stream.failure = Some(error);
        false
      }
    }
  }

  /// Closes the sidecar and returns it, or nothing if no moment was kept: a
  /// recording without moments leaves no file behind.
  pub(crate) fn stop(self: Arc<Self>) -> Option<PathBuf> {
    self.retire();
    let mut stream = self.stream();
    stream.clock.stop();
    let flushed = stream.writer.flush();
    if stream.count == 0 {
      let _ = std::fs::remove_file(&self.path);
      return None;
    }
    if let Err(error) = flushed {
      eprintln!("Could not finish writing moments: {error}");
    }
    Some(self.path.clone())
  }

  pub(crate) fn cancel(self: Arc<Self>) {
    self.retire();
    self.stream().clock.stop();
    let _ = std::fs::remove_file(&self.path);
  }

  /// Stops kind shortcuts reaching this recorder.
  fn retire(self: &Arc<Self>) {
    let mut active = active();
    if active
      .as_ref()
      .is_some_and(|current| Arc::ptr_eq(current, self))
    {
      *active = None;
    }
  }
}

/// Writes a moment of `kind` into the running recording, if there is one
/// that can place it. Whether one was written.
pub(super) fn record(kind: &MomentKind, at: Instant) -> bool {
  let recorder = active().clone();
  recorder.is_some_and(|recorder| recorder.record(kind, at))
}

fn write_line(writer: &mut BufWriter<File>, record: &MomentRecord) -> Result<(), String> {
  serde_json::to_writer(&mut *writer, record).map_err(|error| error.to_string())?;
  writer.write_all(b"\n").map_err(|error| error.to_string())?;
  writer.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
impl MomentRecorder {
  /// A recorder that is not published to the kind shortcuts.
  pub(super) fn detached(path: PathBuf, origin: Arc<OnceLock<Instant>>) -> Self {
    Self::open(path, origin).expect("open moments sidecar")
  }

  pub(super) fn record_for_test(&self, kind: &MomentKind, at: Instant) -> bool {
    self.record(kind, at)
  }

  pub(super) fn finish_for_test(self) -> Option<PathBuf> {
    Arc::new(self).stop()
  }
}
