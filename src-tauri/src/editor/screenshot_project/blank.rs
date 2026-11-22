// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A blank canvas (#41): a screenshot project whose only layer is blank,
//! started from the tray, and the blank layer a canvas is given when its
//! last picture is deleted. A blank layer's picture is all clear, so the
//! canvas shows through it; it holds what is drawn on the canvas as any
//! layer holds its annotations.

use super::*;

/// A new blank canvas's size, in pixels.
const CANVAS_WIDTH: u32 = 1920;
const CANVAS_HEIGHT: u32 = 1080;
/// The largest side a blank layer is made with, in pixels, so a request
/// cannot ask for more memory than a picture of the screen would take.
const MOST_SIDE: u32 = 16_384;

/// Opens the editor on a new screenshot project holding a blank canvas, in
/// place of whatever screenshot it holds, which is kept in its project.
pub(crate) fn open_blank_canvas(app: &AppHandle) {
  let app = app.clone();
  // Off the tray's thread: the empty picture is written into the project.
  tauri::async_runtime::spawn_blocking(move || {
    let item = ScreenshotItem::blank(next_id(&app), CANVAS_WIDTH, CANVAS_HEIGHT);
    let title = crate::screenshots::capture_file_stem(chrono::Local::now().naive_local());
    let opened = keep_layer(&app, None, &title, &item, false).and_then(|project| {
      present_new(
        &app,
        EditorArtifact::Screenshot {
          id: next_id(&app),
          items: vec![item],
          project,
          suggested_file_stem: title,
        },
        Origin::Project,
      )
    });
    if let Err(error) = opened {
      eprintln!("Could not open a blank canvas: {error}");
    }
  });
}

/// Adds a blank layer `width` by `height` pixels to the open screenshot
/// `artifact_id`, as its last picture is deleted, so the canvas keeps a
/// layer for what is drawn on it.
#[tauri::command]
pub async fn add_blank_screenshot_layer(
  app: AppHandle,
  artifact_id: u64,
  width: u32,
  height: u32,
) -> Result<(), String> {
  let (project, _) = workspace::open_project(&app, artifact_id)?;
  let item = ScreenshotItem::blank(
    next_id(&app),
    width.clamp(1, MOST_SIDE),
    height.clamp(1, MOST_SIDE),
  );
  let kept_app = app.clone();
  let item = tauri::async_runtime::spawn_blocking(move || {
    keep_layer(&kept_app, Some(&project), "", &item, false).map(|_| item)
  })
  .await
  .map_err(|error| error.to_string())??;
  {
    let state = app.state::<EditorState>();
    let mut artifact = state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Screenshot { id, items, .. }) = artifact.as_mut() else {
      return Err("That screenshot is no longer open in the editor".to_owned());
    };
    if *id != artifact_id {
      return Err("That screenshot is no longer open in the editor".to_owned());
    }
    items.push(item);
  }
  emit_snapshot(&app, EditorKind::Screenshot);
  Ok(())
}
