// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::overlay;

const PROGRESS_EVENT: &str = "scrolling-capture://progress";
const FINISHED_EVENT: &str = "scrolling-capture://finished";

/// Parking the pointer, settling, and seeking back to the top all read as one
/// undifferentiated wait to someone watching their window get driven around.
pub(super) const WORKING: &str = "working";
pub(super) const CAPTURING: &str = "capturing";
pub(super) const STITCHING: &str = "stitching";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScrollingCaptureProgress {
  phase: &'static str,
}

/// Progress is advisory: a capture must never fail because its overlay went
/// away, so a delivery error is dropped rather than propagated.
pub(super) fn emit(app: &AppHandle, phase: &'static str) {
  let _ = app.emit_to(
    overlay::LABEL,
    PROGRESS_EVENT,
    ScrollingCaptureProgress { phase },
  );
}

/// How the capture ended is not described here: an unalignable seam is stitched
/// best-effort rather than failed, and what remains is reported by the command
/// rejecting. The overlay only needs to know it is over.
pub(super) fn emit_finished(app: &AppHandle) {
  let _ = app.emit_to(overlay::LABEL, FINISHED_EVENT, ());
}
