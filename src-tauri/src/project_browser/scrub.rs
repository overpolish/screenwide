// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scrub strip a card scrubs through under the pointer, so a take can be
//! checked without opening it. The editor composes it, edit and all, so a
//! recording never opened in the editor has none yet.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

/// A project's scrub strip file: its frames side by side, each the same
/// size.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScrubStripFile {
  frame_height: u32,
  frame_width: u32,
  frames: u32,
  path: PathBuf,
}

/// The strip of the project whose manifest is `file`, or None while it has
/// none. Only the picture's header is read, to know its size.
#[tauri::command]
pub fn get_scrub_strip(app: AppHandle, file: PathBuf) -> Option<ScrubStripFile> {
  let path = crate::project::scrub_strip_path(&file);
  let (width, height) = image::image_dimensions(&path).ok()?;
  let frame_width = crate::project::SCRUB_FRAME_WIDTH;
  let frames = width / frame_width;
  if frames == 0 || height == 0 {
    return None;
  }
  // As with the preview, each strip is let through as the browser asks for
  // it rather than opening the asset protocol to every folder.
  app
    .asset_protocol_scope()
    .allow_file(&path)
    .inspect_err(|error| eprintln!("Could not show {}: {error}", path.display()))
    .ok()?;
  Some(ScrubStripFile {
    frame_height: height,
    frame_width,
    frames,
    path,
  })
}
