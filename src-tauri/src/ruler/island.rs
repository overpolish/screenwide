// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The island while the Ruler is up: its tools, which a press picks as a
//! held key picks them, and what the session can do besides measuring. It
//! stands above the Ruler's overlay, so the overlay never takes a press
//! meant for it, and gives the keyboard back after each press so the
//! Ruler's keys keep working.
//!
//! The island shows the session rather than keeping its own state: every
//! Ruler result is checked against what the island was last told
//! (`ruler://island`), and only a change is sent.

use screenwide_imaging::analysis::Tolerance;
use screenwide_osc::protocol::InputPhase;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use super::RulerState;
use crate::app_windows;
use crate::app_windows::island::{self, IslandClaim, IslandSlot};

pub(crate) mod commands;
pub(super) use app_windows::island_overlay::keep_out_of_captures;

/// What the island was last told; none while the Ruler is not on it.
static ISLAND: IslandSlot<RulerIslandState> = IslandSlot::new(IslandClaim::Ruler, "ruler://island");

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

/// The Ruler is starting: the island comes out with its tools at once, while
/// the desktop is frozen and the overlay built, so the start is seen as it
/// happens. Called with the island kept out of captures, so it is not frozen
/// into the desktop.
pub(super) fn come_out(app: &AppHandle) {
  if let Some(state) = app.state::<RulerState>().island_state() {
    ISLAND.tell(app, state);
  }
}

/// The overlay is presented: the island stands above it. On Windows, where
/// there are no window levels, that is by being owned by it.
pub(super) fn stand_over_overlay(app: &AppHandle) {
  if !ISLAND.is_up() {
    return;
  }
  let overlay = super::ruler_windows(app).into_iter().next();
  app_windows::island_overlay::stand_over(app, overlay.as_ref());
  sync(app);
}

/// The Ruler went. The island drops its tools and its place above the
/// overlay, and goes back to whatever else holds it, or tucks away.
pub(super) fn withdraw(app: &AppHandle) {
  if !ISLAND.is_up() {
    return;
  }
  // Released before the Ruler's window closes.
  app_windows::island_overlay::stand_over(app, None);
  ISLAND.withdraw(app);
}

/// A screenshot is taken through the Ruler: the island steps out of the
/// picture at once, and back when it is done.
pub(super) fn set_screenshot_mode(app: &AppHandle, active: bool) {
  if !ISLAND.is_up() {
    return;
  }
  if active {
    island::stand_aside(app, true);
  } else {
    island::come_back(app);
  }
}

/// Tells the island what the session now is, if that changed. Called with
/// every Ruler result.
pub(crate) fn sync(app: &AppHandle) {
  if let Some(next) = app.state::<RulerState>().island_state() {
    ISLAND.update(app, next);
  }
}
