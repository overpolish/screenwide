// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use super::format::{header, note_path, placed_at_us, MomentRecord};
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

/// The recorder kind shortcuts write to now, if a recording is running.
pub(super) fn current() -> Option<Arc<MomentRecorder>> {
  active().clone()
}

pub(crate) struct MomentRecorder {
  path: PathBuf,
  /// The recording's own microphone, which a voice note may share.
  microphone_id: Option<String>,
  stream: Mutex<Stream>,
}

struct Stream {
  clock: SidecarClock,
  count: usize,
  failure: Option<String>,
  writer: Option<BufWriter<File>>,
}

impl MomentRecorder {
  /// Opens the sidecar at `path` on the recording clock `origin`, and makes
  /// it the one kind shortcuts write to. `microphone_id` is the recording's
  /// microphone, if it has one.
  pub(crate) fn start(
    path: PathBuf,
    origin: Arc<OnceLock<Instant>>,
    microphone_id: Option<String>,
  ) -> Result<Arc<Self>, String> {
    let recorder = Arc::new(Self::open(path, origin, microphone_id)?);
    *active() = Some(Arc::clone(&recorder));
    Ok(recorder)
  }

  fn open(
    path: PathBuf,
    origin: Arc<OnceLock<Instant>>,
    microphone_id: Option<String>,
  ) -> Result<Self, String> {
    let file = File::create(&path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(file);
    write_line(&mut writer, &header())?;
    Ok(Self {
      microphone_id,
      path,
      stream: Mutex::new(Stream {
        clock: SidecarClock::new(origin),
        count: 0,
        failure: None,
        writer: Some(writer),
      }),
    })
  }

  fn stream(&self) -> MutexGuard<'_, Stream> {
    self
      .stream
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  pub(super) fn microphone_id(&self) -> Option<&str> {
    self.microphone_id.as_deref()
  }

  /// Where the voice note of the `moment`th moment goes.
  pub(super) fn note_path(&self, moment: usize) -> PathBuf {
    note_path(self.path.parent().unwrap_or(Path::new("")), moment)
  }

  pub(crate) fn pause(&self, at: Instant) {
    self.stream().clock.pause(at);
  }

  pub(crate) fn resume(&self, at: Instant) {
    self.stream().clock.resume(at);
  }

  /// Writes a moment of `kind` for a press at `at`, and says which moment it
  /// is, counting from zero. Nothing is written before the first frame or
  /// while paused, since neither has a place on the recording's timeline.
  pub(super) fn record(&self, kind: &MomentKind, at: Instant) -> Option<usize> {
    let mut stream = self.stream();
    let pressed_us = stream.clock.timestamp_us(at)?;
    let record = MomentRecord::Moment {
      color: kind.color.clone(),
      kind_id: kind.id.clone(),
      name: kind.name.clone(),
      timestamp_us: placed_at_us(pressed_us),
    };
    stream.write(&record).then(|| {
      stream.count += 1;
      stream.count - 1
    })
  }

  /// Notes that the `moment`th moment's voice note is written, at
  /// `duration_ms` long. Too late once the recording has stopped.
  pub(super) fn record_note(&self, moment: usize, duration_ms: u64) -> bool {
    self.stream().write(&MomentRecord::Note {
      duration_ms,
      moment,
    })
  }

  /// Closes the sidecar and returns it, or nothing if no moment was kept: a
  /// recording without moments leaves no file behind.
  pub(crate) fn stop(self: Arc<Self>) -> Option<PathBuf> {
    self.retire();
    let mut stream = self.stream();
    stream.clock.stop();
    let flushed = stream
      .writer
      .take()
      .map_or(Ok(()), |mut writer| writer.flush());
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
    let mut stream = self.stream();
    stream.clock.stop();
    stream.writer = None;
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

impl Stream {
  /// Writes `record` at once: there are few, and each has to survive the app
  /// dying before the recording stops. Whether it was written.
  fn write(&mut self, record: &MomentRecord) -> bool {
    if self.failure.is_some() {
      return false;
    }
    let Some(writer) = self.writer.as_mut() else {
      return false;
    };
    match write_line(writer, record) {
      Ok(()) => true,
      Err(error) => {
        eprintln!("Moments stopped writing: {error}");
        self.failure = Some(error);
        false
      }
    }
  }
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
    Self::open(path, origin, None).expect("open moments sidecar")
  }

  pub(super) fn record_for_test(&self, kind: &MomentKind, at: Instant) -> bool {
    self.record(kind, at).is_some()
  }

  pub(super) fn finish_for_test(self) -> Option<PathBuf> {
    Arc::new(self).stop()
  }
}
