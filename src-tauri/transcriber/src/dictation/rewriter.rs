// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The model that writes each stretch of a streamed dictation again, loaded
//! on a thread of its own as the dictation starts. Loading it took over
//! twice as long as the streaming model, and it is first needed at the
//! first pause, so the streamed words show as soon as their own model has
//! loaded. Stretches ended before it has loaded wait for it; those still
//! waiting when the dictation finishes stay as they were streamed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

use transcribe_cpp::{Model, Session};

/// How often a stretch waiting for the model looks whether the dictation
/// has finished.
const LOOK_EVERY: Duration = Duration::from_millis(20);

pub struct Rewriter {
  path: PathBuf,
  state: State,
}

enum State {
  Loading(Receiver<Result<Session, String>>),
  Loaded(Session),
  /// Could not load; the next dictation tries again.
  Failed,
}

impl Rewriter {
  /// Starts loading the model at `path`.
  pub fn load(path: &Path) -> Self {
    let (loaded, loading) = mpsc::channel();
    let file = path.to_owned();
    std::thread::spawn(move || {
      let session = Model::load(&file)
        .and_then(|model| model.session())
        .map_err(|error| format!("Could not load the dictation model: {error}"));
      // A dictation that has gone has dropped the receiver; the session
      // goes with it.
      let _ = loaded.send(session);
    });
    Self {
      path: path.to_owned(),
      state: State::Loading(loading),
    }
  }

  /// Whether this is the model at `path`, loaded or loading.
  pub fn is_for(&self, path: &Path) -> bool {
    self.path == path && !matches!(self.state, State::Failed)
  }

  /// The model's session, waiting for it to load until `finished` is set;
  /// none once it is, or when the model could not load, which is said on
  /// standard error and leaves the dictation as streamed.
  pub fn session(&mut self, finished: &AtomicBool) -> Option<&mut Session> {
    while let State::Loading(loading) = &self.state {
      match loading.recv_timeout(LOOK_EVERY) {
        Ok(Ok(session)) => self.state = State::Loaded(session),
        Ok(Err(error)) => {
          eprintln!("{error}");
          self.state = State::Failed;
        }
        Err(RecvTimeoutError::Disconnected) => {
          eprintln!("The dictation model stopped loading");
          self.state = State::Failed;
        }
        Err(RecvTimeoutError::Timeout) if finished.load(Ordering::Acquire) => return None,
        Err(RecvTimeoutError::Timeout) => {}
      }
    }
    match &mut self.state {
      State::Loaded(session) => Some(session),
      State::Loading(_) | State::Failed => None,
    }
  }
}
