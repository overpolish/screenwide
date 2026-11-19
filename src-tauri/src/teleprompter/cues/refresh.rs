// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Cues redrawn as their script opens, where the recording or screenshot
//! they show has been edited since: a cue shows its source as it is now.
//! A source moved or put away keeps the picture it had.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Emitter};

use super::picture::{draw, modified, picture_name};
use super::{folder, shown, Cue, PICTURES};
use crate::teleprompter::window::LABEL;

/// Tells the teleprompter a cue was redrawn, with its new picture.
const REDRAWN_EVENT: &str = "teleprompter://cue-redrawn";

/// The scripts whose cues are being redrawn, so a script read twice over,
/// as a page in development reads it, is redrawn once.
static REDRAWING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Redraws, one after another and off the command, each of `cues` of the
/// script `script` whose source was edited after its picture was made.
pub(super) fn redraw_stale(app: &AppHandle, script: PathBuf, cues: Vec<Cue>) {
  {
    let mut redrawing = REDRAWING.lock().unwrap_or_else(PoisonError::into_inner);
    if redrawing.contains(&script) {
      return;
    }
    redrawing.push(script.clone());
  }
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    for cue in cues {
      if let Err(error) = redraw_if_stale(&app, &script, cue).await {
        eprintln!("Could not redraw a cue: {error}");
      }
    }
    REDRAWING
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .retain(|redrawn| *redrawn != script);
  });
}

async fn redraw_if_stale(
  app: &AppHandle,
  script: &std::path::Path,
  cue: Cue,
) -> Result<(), String> {
  let pictures = folder(script)?.join(PICTURES);
  let Some(made) = modified(&pictures.join(&cue.image)) else {
    return Ok(());
  };
  // A recording's edit is saved in its manifest; a screenshot as edited in
  // its large still. A recording's cue made before cues kept their moment
  // cannot be redrawn.
  let edited = match (cue.played_ms, cue.source_ms) {
    (Some(_), Some(_)) => modified(&cue.source),
    (Some(_), None) => None,
    (None, _) => modified(&screenwide_project::still_path(&cue.source)),
  };
  if !edited.is_some_and(|edited| edited > made) {
    return Ok(());
  }
  let image = format!("{}.jpg", picture_name());
  draw(app, &cue.source, cue.source_ms, pictures.join(&image)).await?;
  let redrawn = shown(app, script, Cue { image, ..cue })?;
  app
    .emit_to(LABEL, REDRAWN_EVENT, &redrawn)
    .map_err(|error| error.to_string())
}
