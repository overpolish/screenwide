// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while a scrolling capture runs: it comes out at the top of the
//! screen and says what the capture is doing, as it drives the user's window
//! for several seconds, and how to stop it. It is kept out of every frame the
//! capture takes, so it may stand over the region being photographed.
//!
//! The island shows the capture rather than keeping its own state: each
//! change is sent (`scrolling-capture://island`), and only a change.

use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

use crate::app_windows::island::{IslandClaim, IslandSlot};
use crate::app_windows::island_overlay;

/// What the capture is doing. Parking the pointer, settling, and seeking back
/// to the top all read as one undifferentiated wait to someone watching their
/// window get driven around, so they are one phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ScrollingCapturePhase {
  Working,
  Capturing,
  /// The frames are put together; past this point it cannot be stopped.
  Stitching,
  /// The capture is over and the island is about to go.
  Finishing,
}

/// What the island shows of a scrolling capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScrollingCaptureIslandState {
  pub phase: ScrollingCapturePhase,
  /// Whether Escape was claimed for the capture, so the island only names a
  /// key that works.
  pub escape_claimed: bool,
}

/// What the island was last told; none while no capture is on it.
static ISLAND: IslandSlot<ScrollingCaptureIslandState> =
  IslandSlot::new(IslandClaim::ScrollingCapture, "scrolling-capture://island");

/// The capture is starting: the island comes out at the top of the screen
/// and is kept out of captures. Kept out after it is shown, as showing a
/// window on Windows puts back the persistent capture preference.
pub(super) async fn present(app: &AppHandle, escape_claimed: bool) {
  ISLAND.tell(
    app,
    ScrollingCaptureIslandState {
      phase: ScrollingCapturePhase::Working,
      escape_claimed,
    },
  );
  island_overlay::keep_out_of_captures(app, true).await;
}

/// The capture moved on to `phase`.
pub(super) fn phase(app: &AppHandle, phase: ScrollingCapturePhase) {
  if let Some(state) = ISLAND.current() {
    ISLAND.update(app, ScrollingCaptureIslandState { phase, ..state });
  }
}

/// The capture is over: the island says so as it tucks away, or as it goes
/// back to the recording under way, rather than spinning on a phase that has
/// ended, and captures may see it again. How it ended is not said here: an
/// unalignable seam is stitched best-effort rather than failed, and what
/// remains is reported by the command rejecting.
pub(super) async fn withdraw(app: &AppHandle) {
  if !ISLAND.is_up() {
    return;
  }
  phase(app, ScrollingCapturePhase::Finishing);
  ISLAND.withdraw(app);
  island_overlay::keep_out_of_captures(app, false).await;
}

#[tauri::command]
pub fn scrolling_capture_island_state() -> Option<ScrollingCaptureIslandState> {
  ISLAND.current()
}

/// Cancel pressed on the island: the capture stops at its next step, as it
/// does for Escape.
#[tauri::command]
pub fn cancel_scrolling_capture() {
  super::cancel::request();
}
