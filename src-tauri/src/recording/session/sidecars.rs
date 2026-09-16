// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lifecycle coordination for the optional cursor, keyboard and live
//! annotation sidecars.

use std::{
  path::PathBuf,
  sync::{Arc, OnceLock},
  time::Instant,
};

use super::super::{
  cursor::{CursorRecorder, CursorSource},
  keyboard::KeyboardRecorder,
};
use crate::annotate::live_clips::AnnotationRecorder;
use crate::editor::annotations::timing::RecordingAnnotationClip;

/// Which sidecars a starting recording asks for.
pub(super) struct SidecarPlan {
  pub cursor_path: Option<PathBuf>,
  pub cursor_source: Option<CursorSource>,
  pub keyboard_path: Option<PathBuf>,
  /// Live marks are recorded in the desktop's coordinate space, so they ride
  /// with the modes that record the desktop.
  pub records_annotations: bool,
}

pub(super) struct RecordingSidecars {
  pub annotations: Option<AnnotationRecorder>,
  pub cursor: Option<CursorRecorder>,
  pub keyboard: Option<KeyboardRecorder>,
}

pub(super) struct StoppedSidecars {
  pub annotation_clips: Vec<RecordingAnnotationClip>,
  pub cursor: Result<Option<PathBuf>, String>,
  pub keyboard: Result<Option<PathBuf>, String>,
}

impl RecordingSidecars {
  pub(super) fn start(plan: SidecarPlan, origin: Arc<OnceLock<Instant>>) -> Result<Self, String> {
    let SidecarPlan {
      cursor_path,
      cursor_source,
      keyboard_path,
      records_annotations,
    } = plan;
    let annotation_source = records_annotations.then(|| cursor_source.clone()).flatten();
    let cursor = match (cursor_path, cursor_source) {
      (Some(path), Some(source)) => Some(CursorRecorder::start(path, origin.clone(), source)?),
      (None, _) => None,
      (Some(_), None) => {
        return Err("The capture source has no cursor coordinate space".to_owned())
      }
    };
    let keyboard = match keyboard_path {
      Some(path) => match KeyboardRecorder::start(path, origin.clone()) {
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
    // Last, because it cannot fail and so needs no unwinding of its own.
    let annotations = annotation_source.map(|source| AnnotationRecorder::start(origin, source));
    Ok(Self {
      annotations,
      cursor,
      keyboard,
    })
  }

  pub(super) fn pause(&self, at: Instant) {
    if let Some(annotations) = &self.annotations {
      annotations.pause(at);
    }
    if let Some(cursor) = &self.cursor {
      cursor.pause(at);
    }
    if let Some(keyboard) = &self.keyboard {
      keyboard.pause(at);
    }
  }

  pub(super) fn resume(&self, at: Instant) {
    if let Some(annotations) = &self.annotations {
      annotations.resume(at);
    }
    if let Some(cursor) = &self.cursor {
      cursor.resume(at);
    }
    if let Some(keyboard) = &self.keyboard {
      keyboard.resume(at);
    }
  }

  pub(super) fn stop(self, stopped_at: Instant) -> StoppedSidecars {
    StoppedSidecars {
      annotation_clips: self
        .annotations
        .map(|annotations| annotations.stop(stopped_at))
        .unwrap_or_default(),
      cursor: self.cursor.map(CursorRecorder::stop).transpose(),
      keyboard: self.keyboard.map(KeyboardRecorder::stop).transpose(),
    }
  }

  pub(super) fn cancel(self) {
    if let Some(annotations) = self.annotations {
      annotations.cancel();
    }
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
