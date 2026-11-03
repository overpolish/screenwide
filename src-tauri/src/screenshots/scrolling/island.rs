// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island while a scrolling capture runs: it comes out at the top of the
//! screen and says what the capture is doing, as it drives the user's window
//! for several seconds, and how to stop it. It is kept out of every frame the
//! capture takes, so it may stand over the region being photographed.
//!
//! The island shows the capture rather than keeping its own state: each
//! change is sent (`scrolling-capture://island`), and only a change.

use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, island_overlay, WindowLabel};

const EVENT: &str = "scrolling-capture://island";

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
static SHOWN: Mutex<Option<ScrollingCaptureIslandState>> = Mutex::new(None);

fn shown() -> std::sync::MutexGuard<'static, Option<ScrollingCaptureIslandState>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

fn tell(app: &AppHandle, state: Option<ScrollingCaptureIslandState>) {
  let mut shown = shown();
  if *shown == state {
    return;
  }
  *shown = state;
  drop(shown);
  // Advisory: a capture must never fail because the island did not hear.
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, state);
}

/// The capture is starting: the island comes out at the top of the screen
/// and is kept out of captures. Kept out after it is shown, as showing a
/// window on Windows puts back the persistent capture preference.
pub(super) async fn present(app: &AppHandle, escape_claimed: bool) {
  tell(
    app,
    Some(ScrollingCaptureIslandState {
      phase: ScrollingCapturePhase::Working,
      escape_claimed,
    }),
  );
  let shown = app_windows::island::place(app, IslandPlacement::Top)
    .and_then(|()| app_windows::island::show(app));
  if let Err(error) = shown {
    eprintln!("Could not show the island for a scrolling capture: {error}");
  }
  island_overlay::keep_out_of_captures(app, true).await;
}

/// The capture moved on to `phase`.
pub(super) fn phase(app: &AppHandle, phase: ScrollingCapturePhase) {
  let current = *shown();
  if let Some(state) = current {
    tell(app, Some(ScrollingCaptureIslandState { phase, ..state }));
  }
}

/// The capture is over: the island says so as it tucks away, rather than
/// spinning on a phase that has ended, and captures may see it again. How it
/// ended is not said here: an unalignable seam is stitched best-effort rather
/// than failed, and what remains is reported by the command rejecting.
pub(super) async fn withdraw(app: &AppHandle) {
  if shown().is_none() {
    return;
  }
  phase(app, ScrollingCapturePhase::Finishing);
  tell(app, None);
  if let Err(error) = app_windows::island::hide(app) {
    eprintln!("Could not put the island away after a scrolling capture: {error}");
  }
  island_overlay::keep_out_of_captures(app, false).await;
}

#[tauri::command]
pub fn scrolling_capture_island_state() -> Option<ScrollingCaptureIslandState> {
  *shown()
}

/// Cancel pressed on the island: the capture stops at its next step, as it
/// does for Escape.
#[tauri::command]
pub fn cancel_scrolling_capture() {
  super::cancel::request();
}
