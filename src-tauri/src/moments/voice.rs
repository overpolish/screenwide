// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The voice note a held kind shortcut records.
//!
//! The microphone opens as the key goes down, so it is listening by the time
//! the hold is long enough to count, and a tap closes it again with nothing
//! kept. A note shares the recording's own microphone when that is the one
//! it would use, rather than opening the device twice; either way the
//! recording's microphone is held out of the note (`recording::note_gate`).

use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::AppHandle;

use super::note_audio::{write_wav, NoteAudio};
use super::recorder::MomentRecorder;
use super::settings;
use crate::recording::microphone::Source;
use crate::recording::note_gate::{self, NoteTee, NOTE_THRESHOLD};

/// How long after the key comes up the microphone is still read: its last
/// buffers arrive a little after the moment they were captured.
const TRAILING_AUDIO: Duration = Duration::from_millis(150);

/// What a press does about a note, as the dock shows it.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum Voice {
  /// Listening; held long enough, it is a note.
  Recording,
  /// Notes are on, but there is no microphone to take one from.
  Unavailable,
  Off,
}

struct Capture {
  audio: Arc<Mutex<NoteAudio>>,
  /// The note's own microphone, when it does not share the recording's.
  microphone: Option<OwnMicrophone>,
  moment: usize,
  recorder: Arc<MomentRecorder>,
  start: Instant,
  /// Whether the recording's microphone is being held out of the note.
  gated: bool,
}

static CAPTURE: Mutex<Option<Capture>> = Mutex::new(None);

fn capture() -> MutexGuard<'static, Option<Capture>> {
  CAPTURE
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn audio(audio: &Mutex<NoteAudio>) -> MutexGuard<'_, NoteAudio> {
  audio
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Starts listening for the note of the `moment`th moment, pressed at `at`.
pub(super) fn begin(
  app: &AppHandle,
  recorder: Arc<MomentRecorder>,
  moment: usize,
  at: Instant,
) -> Voice {
  let settings = settings::current();
  if !settings.voice_notes {
    return Voice::Off;
  }
  let mut current = capture();
  // One note at a time: a second key held over the first places its moment
  // alone.
  if current.is_some() {
    return Voice::Off;
  }
  let recording_microphone = recorder.microphone_id().map(str::to_owned);
  let shares = match settings.note_microphone.as_deref() {
    None => recording_microphone.is_some(),
    Some(id) => recording_microphone.as_deref() == Some(id),
  };
  let note = Arc::new(Mutex::new(NoteAudio::default()));
  let microphone = if shares {
    let heard = Arc::clone(&note);
    let tee: NoteTee = Arc::new(move |samples, channels, sample_rate, captured_at| {
      audio(&heard).take(samples, channels, sample_rate, captured_at);
    });
    note_gate::press(at, Some(tee));
    None
  } else {
    // Opening a microphone the app may not use would ask for permission in
    // the middle of a recording; Settings asks when notes are switched on.
    if !crate::permissions::cached_snapshot(app).microphone.granted {
      return Voice::Unavailable;
    }
    let gated = recording_microphone.is_some();
    if gated {
      note_gate::press(at, None);
    }
    Some(OwnMicrophone::start(
      settings.note_microphone,
      Arc::clone(&note),
      gated,
    ))
  };
  *current = Some(Capture {
    audio: note,
    gated: recording_microphone.is_some(),
    microphone,
    moment,
    recorder,
    start: at,
  });
  Voice::Recording
}

/// The key came up at `at`: a tap lets the note go, a hold keeps it.
pub(super) fn end(at: Instant) {
  let Some(capture) = capture().take() else {
    return;
  };
  if capture.gated {
    note_gate::release(at);
  }
  audio(&capture.audio).end = Some(at);
  let held = at.saturating_duration_since(capture.start);
  // Off the shortcut's callback: a microphone still opening is waited for,
  // and for a note the last buffers are too, and the file is written, before
  // the note is recorded.
  std::thread::spawn(move || {
    if held < NOTE_THRESHOLD {
      if let Some(microphone) = capture.microphone {
        microphone.stop();
      }
      return;
    }
    std::thread::sleep(TRAILING_AUDIO);
    if let Some(microphone) = capture.microphone {
      microphone.stop();
    }
    keep(&capture.recorder, capture.moment, &audio(&capture.audio));
  });
}

fn keep(recorder: &MomentRecorder, moment: usize, note: &NoteAudio) {
  if note.failed || note.samples.is_empty() {
    return;
  }
  let path = recorder.note_path(moment);
  if let Err(error) = write_wav(&path, note.sample_rate, &note.samples) {
    eprintln!("Could not keep a voice note: {error}");
    return;
  }
  // The recording stopped while the key was held: the note has no moment
  // file to be listed in, so it is not kept.
  if !recorder.record_note(moment, note.duration_ms()) {
    let _ = std::fs::remove_file(path);
  }
}

/// A microphone opened for one note, on a thread of its own so the stream
/// is opened and closed where it lives.
struct OwnMicrophone {
  stop: mpsc::Sender<()>,
  thread: JoinHandle<()>,
}

impl OwnMicrophone {
  fn start(device_id: Option<String>, note: Arc<Mutex<NoteAudio>>, gated: bool) -> Self {
    let (stop, stopped) = mpsc::channel::<()>();
    let thread = std::thread::spawn(move || {
      let fail = |error: String| {
        eprintln!("Could not open the microphone for a voice note: {error}");
        audio(&note).failed = true;
        // Nothing is recorded, so the recording keeps its own microphone.
        if gated {
          note_gate::cancel();
        }
      };
      let source = match Source::resolve_or_default(device_id.as_deref()) {
        Ok(source) => source,
        Err(error) => return fail(error),
      };
      let format = source.format();
      let heard = Arc::clone(&note);
      let stream = source.start(
        Arc::new(move |buffer| {
          audio(&heard).take(
            &buffer.samples,
            format.channels,
            format.sample_rate,
            buffer.captured_at,
          );
        }),
        Arc::new(|error| eprintln!("The voice note microphone failed: {error}")),
      );
      match stream {
        Ok(stream) => {
          let _ = stopped.recv();
          drop(stream);
        }
        Err(error) => fail(error),
      }
    });
    Self { stop, thread }
  }

  fn stop(self) {
    let _ = self.stop.send(());
    let _ = self.thread.join();
  }
}
