// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Zoom scenes made from what the recording's cursor and keys show of where
//! the work happened. Clicks, drags and typing are gathered into beats, the
//! moments another app came to the front split them, and the beats are
//! planned into shots across the whole recording at once. Each shot becomes
//! an ordinary full scene that zooms the screen, marked as made here so that
//! making them again replaces only these.

mod beats;
mod shots;
mod signals;
#[cfg(test)]
mod tests;

use std::path::Path;

use screenwide_records::cursor::CursorRecord;
use screenwide_records::keyboard::KeyboardRecord;
use screenwide_scenes::{RecordingSceneClip, RecordingScenePreset};
use serde::Deserialize;
use signals::Signals;

/// The part of the screen's picture the composition shows, as shares of the
/// picture. A scene's screen framing is measured against this part, so the
/// cursor is too.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VisibleArea {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

impl VisibleArea {
  pub const WHOLE: Self = Self {
    x: 0.0,
    y: 0.0,
    width: 1.0,
    height: 1.0,
  };
}

/// A rectangle in shares of the visible picture.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
  left: f64,
  top: f64,
  right: f64,
  bottom: f64,
}

impl Area {
  fn point(x: f64, y: f64) -> Self {
    Self {
      left: x,
      top: y,
      right: x,
      bottom: y,
    }
  }

  fn union(self, other: Self) -> Self {
    Self {
      left: self.left.min(other.left),
      top: self.top.min(other.top),
      right: self.right.max(other.right),
      bottom: self.bottom.max(other.bottom),
    }
  }

  /// The longer side: a zoom shows as much of the picture's height as of its
  /// width, so the longer side is what has to fit.
  fn span(self) -> f64 {
    (self.right - self.left).max(self.bottom - self.top)
  }

  fn centre(self) -> (f64, f64) {
    (
      (self.left + self.right) / 2.0,
      (self.top + self.bottom) / 2.0,
    )
  }

  fn contains(self, x: f64, y: f64, margin: f64) -> bool {
    x >= self.left - margin
      && x <= self.right + margin
      && y >= self.top - margin
      && y <= self.bottom + margin
  }
}

/// The auto zooms for the recording whose cursor and keys were saved at
/// these paths. Keys only hold a zoom on, so a key file that cannot be read
/// still leaves the zooms the clicks call for.
pub fn plan(
  cursor_path: &Path,
  keyboard_path: Option<&Path>,
  duration_ms: u64,
  visible: VisibleArea,
) -> Result<Vec<RecordingSceneClip>, String> {
  let cursor = screenwide_records::cursor::read(cursor_path)?;
  let keyboard = keyboard_path
    .and_then(|path| screenwide_records::keyboard::read(path).ok())
    .unwrap_or_default();
  Ok(plan_records(&cursor, &keyboard, duration_ms, visible))
}

/// The auto zooms for a recording's cursor and keys already in hand, such as
/// a pointer scripted for a preview.
pub fn plan_records(
  cursor: &[CursorRecord],
  keyboard: &[KeyboardRecord],
  duration_ms: u64,
  visible: VisibleArea,
) -> Vec<RecordingSceneClip> {
  clips(&Signals::read(cursor, keyboard, visible), duration_ms)
}

fn clips(signals: &Signals, duration_ms: u64) -> Vec<RecordingSceneClip> {
  let beats = beats::group(signals);
  shots::plan(&beats, signals, duration_ms)
    .into_iter()
    .map(|shot| RecordingSceneClip {
      // Shots never share a start, so their ids never meet; a scene of your
      // own carries a random one.
      id: format!("auto-zoom-{}", shot.start_ms),
      start_ms: shot.start_ms,
      end_ms: shot.end_ms,
      preset: RecordingScenePreset::Full,
      screen: Some(shot.framing),
      camera: None,
      boxes: None,
      radius: None,
      inset: None,
      rotation: None,
      variant: None,
      follow_face: None,
      auto: true,
    })
    .collect()
}
