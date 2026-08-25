// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lifecycle coordination for the optional cursor and keyboard sidecars.

use std::{
  path::PathBuf,
  sync::{Arc, OnceLock},
  time::Instant,
};

use super::super::{
  cursor::{CursorRecorder, CursorSource},
  keyboard::KeyboardRecorder,
};

pub(super) struct RecordingSidecars {
  pub cursor: Option<CursorRecorder>,
  pub keyboard: Option<KeyboardRecorder>,
}

pub(super) struct StoppedSidecars {
  pub cursor: Result<Option<PathBuf>, String>,
  pub keyboard: Result<Option<PathBuf>, String>,
}

impl RecordingSidecars {
  pub(super) fn start(
    cursor_path: Option<PathBuf>,
    cursor_source: Option<CursorSource>,
    keyboard_path: Option<PathBuf>,
    origin: Arc<OnceLock<Instant>>,
  ) -> Result<Self, String> {
    let cursor = match (cursor_path, cursor_source) {
      (Some(path), Some(source)) => Some(CursorRecorder::start(path, origin.clone(), source)?),
      (None, _) => None,
      (Some(_), None) => {
        return Err("The capture source has no cursor coordinate space".to_owned())
      }
    };
    let keyboard = match keyboard_path {
      Some(path) => match KeyboardRecorder::start(path, origin) {
        Ok(keyboard) => Some(keyboard),
        Err(error) => {
          if let Some(cursor) = cursor {
            cursor.cancel();
          }
          return Err(error);
        }
      },
      None => None,
    };
    Ok(Self { cursor, keyboard })
  }

  pub(super) fn pause(&self, at: Instant) {
    if let Some(cursor) = &self.cursor {
      cursor.pause(at);
    }
    if let Some(keyboard) = &self.keyboard {
      keyboard.pause(at);
    }
  }

  pub(super) fn resume(&self, at: Instant) {
    if let Some(cursor) = &self.cursor {
      cursor.resume(at);
    }
    if let Some(keyboard) = &self.keyboard {
      keyboard.resume(at);
    }
  }

  pub(super) fn stop(self) -> StoppedSidecars {
    StoppedSidecars {
      cursor: self.cursor.map(CursorRecorder::stop).transpose(),
      keyboard: self.keyboard.map(KeyboardRecorder::stop).transpose(),
    }
  }

  pub(super) fn cancel(self) {
    if let Some(cursor) = self.cursor {
      cursor.cancel();
    }
    if let Some(keyboard) = self.keyboard {
      keyboard.cancel();
    }
  }
}

pub(super) fn remove_stopped(sidecars: &StoppedSidecars) {
  if let Ok(Some(path)) = &sidecars.cursor {
    let _ = std::fs::remove_file(path);
  }
  if let Ok(Some(path)) = &sidecars.keyboard {
    let _ = std::fs::remove_file(path);
  }
}
