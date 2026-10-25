// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while the Ruler is up: its tools, which a press picks as a
//! held key picks them, and what the session can do besides measuring. It
//! stands above the Ruler's overlay, so the overlay never takes a press
//! meant for it, and gives the keyboard back after each press so the
//! Ruler's keys keep working.
//!
//! The island shows the session rather than keeping its own state: every
//! Ruler result is checked against what the island was last told
//! (`ruler://island`), and only a change is sent.

use std::sync::{Mutex, PoisonError};

use screenwide_imaging::analysis::Tolerance;
use screenwide_osc::protocol::InputPhase;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use super::RulerState;
use crate::app_windows::{self, WindowLabel};

pub(crate) mod commands;
mod platform;
pub(super) use platform::keep_out_of_captures;

const EVENT: &str = "ruler://island";

/// What a click on the desktop does: measure, the default, or one of the
/// gestures a held key otherwise picks.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RulerTool {
  Measure,
  VerticalGuide,
  HorizontalGuide,
  Radius,
}

/// How faint an edge the Ruler snaps to.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RulerTolerance {
  ClearEdges,
  Balanced,
  SubtleEdges,
}

impl From<Tolerance> for RulerTolerance {
  fn from(tolerance: Tolerance) -> Self {
    match tolerance {
      Tolerance::ClearEdges => Self::ClearEdges,
      Tolerance::Balanced => Self::Balanced,
      Tolerance::SubtleEdges => Self::SubtleEdges,
    }
  }
}

/// What the island shows of a Ruler session.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RulerIslandState {
  pub tool: RulerTool,
  pub tolerance: RulerTolerance,
  pub centerlines: bool,
  pub crosshair: bool,
  pub can_undo: bool,
  /// There is a measurement to copy.
  pub can_copy: bool,
}

/// What a press on the island does once, as its key does.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RulerIslandAction {
  ToggleCenterlines,
  ToggleCrosshair,
  Undo,
  CopyMeasurement,
}

impl RulerIslandAction {
  const fn phase(self) -> InputPhase {
    match self {
      Self::ToggleCenterlines => InputPhase::RulerToggleCenterlines,
      Self::ToggleCrosshair => InputPhase::RulerToggleCrosshair,
      Self::Undo => InputPhase::RulerUndo,
      Self::CopyMeasurement => InputPhase::RulerCopyMeasurement,
    }
  }
}

/// What the island was last told; none while the Ruler is not on it.
static SHOWN: Mutex<Option<RulerIslandState>> = Mutex::new(None);

fn shown() -> std::sync::MutexGuard<'static, Option<RulerIslandState>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The Ruler is up: the island comes out with its tools, above the overlay.
/// Called once the overlay is presented, so on Windows, where there are no
/// window levels, raising the island puts it above the overlay.
pub(super) fn present(app: &AppHandle) {
  let Some(state) = app.state::<RulerState>().island_state() else {
    return;
  };
  *shown() = Some(state);
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, Some(state));
  if let Err(error) = app_windows::island::set_above_overlays(app, true) {
    eprintln!("Could not raise the island over the Ruler: {error}");
  }
  #[cfg(target_os = "windows")]
  platform::own_by_ruler(app, true);
  #[cfg(target_os = "macos")]
  platform::exempt_from_crosshair(app, true);
  if let Err(error) = app_windows::island::show(app) {
    eprintln!("Could not show the island for the Ruler: {error}");
  }
}

/// The Ruler went. The island drops its tools and its place above the
/// overlay, and goes unless the recording or its controls keep it.
pub(super) fn withdraw(app: &AppHandle) {
  if shown().take().is_none() {
    return;
  }
  let _ = app.emit_to(
    WindowLabel::Island.as_str(),
    EVENT,
    None::<RulerIslandState>,
  );
  // Released before the Ruler's window closes: a window takes the windows it
  // owns with it when it is destroyed.
  #[cfg(target_os = "windows")]
  platform::own_by_ruler(app, false);
  #[cfg(target_os = "macos")]
  platform::exempt_from_crosshair(app, false);
  if let Err(error) = app_windows::island::set_above_overlays(app, false) {
    eprintln!("Could not return the island to its level: {error}");
  }
  if crate::recording::is_idle(app) && !app_windows::is_recording_ui_visible() {
    if let Err(error) = app_windows::island::hide(app) {
      eprintln!("Could not hide the island after the Ruler: {error}");
    }
  }
}

/// A screenshot is taken through the Ruler: the island steps out of the
/// picture at once, and back when it is done.
pub(super) fn set_screenshot_mode(app: &AppHandle, active: bool) {
  if shown().is_none() {
    return;
  }
  let result = if active {
    app_windows::island::hide_at_once(app)
  } else {
    app_windows::island::show(app)
  };
  if let Err(error) = result {
    eprintln!("Could not move the island for a Ruler screenshot: {error}");
  }
}

/// Tells the island what the session now is, if that changed. Called with
/// every Ruler result, so it reads the session and compares before it sends.
pub(crate) fn sync(app: &AppHandle) {
  let mut shown = shown();
  let Some(current) = *shown else {
    return;
  };
  let Some(next) = app.state::<RulerState>().island_state() else {
    return;
  };
  if next == current {
    return;
  }
  *shown = Some(next);
  drop(shown);
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, Some(next));
}
