// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the preview is told about the frame a move actually achieved. An
//! application with its own size limits cannot fill the region it was thrown
//! at, and the preview would otherwise keep promising a fill that never
//! happens. Its own event, like the icon's, so the live input events stay a
//! `Copy` enum, and its own module so the command layer keeps its headroom.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::windows::WindowLabel;

/// A rectangle in work-area fractions: the space the preview's mini-map draws
/// in, so it can render this without knowing anything about the monitor.
#[derive(Clone, Copy, Serialize)]
pub(super) struct FitRect {
  pub(super) x: f64,
  pub(super) y: f64,
  pub(super) width: f64,
  pub(super) height: f64,
}

/// What `glide://fit` carries. `fits` says whether the window took the
/// destination's size; `actual` rides along either way, so the preview never
/// has to guess what it is drawing.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GlideFitEvent {
  pub(super) session_id: u64,
  pub(super) fits: bool,
  pub(super) actual: FitRect,
}

/// Sends one settled frame on. Fire-and-forget, from whichever thread settled
/// the move: an answer that arrives after the session ended carries an id the
/// preview no longer matches, so it lands nowhere.
pub(super) fn emit_fit(app: &AppHandle, event: GlideFitEvent) -> Result<(), String> {
  app
    .emit_to(WindowLabel::Glide.as_str(), "glide://fit", event)
    .map_err(|error| error.to_string())
}
