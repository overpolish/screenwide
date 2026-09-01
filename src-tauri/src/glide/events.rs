// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The presentation events shared by native Glide sessions and the preview.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::core::GlideDetection;
use crate::windows::WindowLabel;

#[derive(Clone, Copy, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub(super) enum GlideInputEvent {
  Start {
    session_id: u64,
  },
  /// A state-machine result produced natively from a normalized input sample.
  Detection {
    detection: GlideDetection,
  },
  /// Carries the anchor the session was pinned to, so a gesture the detector
  /// only commits on lift still knows which window it was aimed at. A cancelled
  /// end closes the session without committing what it had armed.
  End {
    anchor_x: f64,
    anchor_y: f64,
    cancelled: bool,
  },
}

pub(super) fn emit(app: &AppHandle, event: GlideInputEvent) -> Result<(), String> {
  app
    .emit_to(WindowLabel::Glide.as_str(), "glide://input", event)
    .map_err(|error| error.to_string())
}

pub(super) fn detection(app: &AppHandle, result: GlideDetection) {
  let _ = emit(app, GlideInputEvent::Detection { detection: result });
}
