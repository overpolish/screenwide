// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The target application's icon, on its own event and its own thread. Its own
//! event because a path field would cost `GlideInputEvent` its `Copy` derive;
//! its own thread because the first glide of an application pays for an AppKit
//! image and a PNG encode, which neither the main thread nor the event tap can
//! afford to wait on. Every later glide of it is a cache hit.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::windows::WindowLabel;

/// What `glide://icon` carries. A miss reports too, so the preview settles on
/// having no icon rather than waiting for one that is never coming.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GlideIconEvent {
  pub(super) session_id: u64,
  pub(super) icon_path: Option<PathBuf>,
}

/// Resolves the icon of the process behind the session's window and sends it
/// on. Fire-and-forget: an answer that arrives after the session ended carries
/// an id the preview no longer matches, so it lands nowhere. A window whose
/// owner could not be read still reports, as a miss.
pub(super) fn spawn_icon_lookup(app: &AppHandle, session_id: u64, pid: Option<u32>) {
  let app = app.clone();
  let _ = std::thread::Builder::new()
    .name("glide-app-icon".to_owned())
    .spawn(move || {
      let icon_path = pid.and_then(|pid| {
        crate::recording_sources::application_icon_cache_dir(&app)
          .ok()
          .and_then(|cache_dir| crate::recording_sources::app_icon(&cache_dir, pid))
      });
      if let Err(error) = emit_icon(&app, session_id, icon_path) {
        eprintln!("Could not send the Glide app icon: {error}");
      }
    });
}

fn emit_icon(app: &AppHandle, session_id: u64, icon_path: Option<PathBuf>) -> Result<(), String> {
  app
    .emit_to(
      WindowLabel::Glide.as_str(),
      "glide://icon",
      GlideIconEvent {
        session_id,
        icon_path,
      },
    )
    .map_err(|error| error.to_string())
}
