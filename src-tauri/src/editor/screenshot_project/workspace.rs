// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The screenshot editor's canvas and layers, saved in the project.
//!
//! The window names layers by ids that last one session, so the project keeps
//! them by their place in the stack instead: the canvas, then each layer's
//! settings in the order its pictures were taken.

use serde_json::{json, Value};

use super::*;

/// The open screenshot's project and its layers' ids, back to front, if it is
/// `artifact_id`.
fn open_project(app: &AppHandle, artifact_id: u64) -> Result<(PathBuf, Vec<u64>), String> {
  let state = app.state::<EditorState>();
  let artifact = state
    .screenshot
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  match artifact.as_ref() {
    Some(EditorArtifact::Screenshot {
      id, items, project, ..
    }) if *id == artifact_id => Ok((project.clone(), items.iter().map(|item| item.id).collect())),
    _ => Err("That screenshot is no longer open in the editor".to_owned()),
  }
}

/// The window's workspace as the project keeps it.
fn stored(mut workspace: Value, ids: &[u64]) -> Value {
  let items = workspace
    .as_object_mut()
    .and_then(|canvas| canvas.remove("items"))
    .unwrap_or_default();
  let layers = ids
    .iter()
    .map(|id| {
      items
        .as_array()
        .and_then(|items| {
          items
            .iter()
            .find(|item| item.get("id").and_then(Value::as_u64) == Some(*id))
        })
        .and_then(|item| item.get("output").cloned())
        .unwrap_or(Value::Null)
    })
    .collect::<Vec<_>>();
  json!({ "canvas": workspace, "layers": layers })
}

/// The project's workspace as the window takes it, its layers named by this
/// session's `ids`. A layer added since it was saved has no settings yet,
/// and is laid out as a new one is.
fn restored(saved: Value, ids: &[u64]) -> Option<Value> {
  let mut canvas = saved.get("canvas")?.clone();
  let layers = saved.get("layers")?.as_array()?;
  let items = ids
    .iter()
    .zip(layers)
    .filter(|(_, output)| !output.is_null())
    .map(|(id, output)| json!({ "id": id, "output": output }))
    .collect::<Vec<_>>();
  canvas
    .as_object_mut()?
    .insert("items".to_owned(), Value::Array(items));
  Some(canvas)
}

/// The workspace saved in the screenshot project `project`, for a window
/// whose layers are `ids`.
pub(crate) fn restored_workspace(project: &Path, ids: &[u64]) -> Option<Value> {
  restored(super::super::project_look::saved_look(project)?, ids)
}

/// Saves the open screenshot's canvas and layers in its project.
#[tauri::command]
pub async fn set_screenshot_project_workspace(
  app: AppHandle,
  artifact_id: u64,
  revision: u64,
  workspace: Value,
) -> Result<(), String> {
  let (project, ids) = open_project(&app, artifact_id)?;
  // Off the async runtime: a background picture is copied in the first time
  // it is saved.
  tauri::async_runtime::spawn_blocking(move || {
    super::super::project_look::save_look(&project, revision, stored(workspace, &ids))
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Composes the open screenshot as the window shows it and keeps it, smaller,
/// as its project's preview.
#[tauri::command]
pub async fn save_screenshot_project_still(
  app: AppHandle,
  artifact_id: u64,
  screenshot_output: ScreenshotWorkspaceOutputSettings,
) -> Result<(), String> {
  // Copied out so the editor is not held while the picture is composed.
  let (project, items) = {
    let state = app.state::<EditorState>();
    let artifact = state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    match artifact.as_ref() {
      Some(EditorArtifact::Screenshot {
        id, items, project, ..
      }) if *id == artifact_id => (project.clone(), items.clone()),
      _ => return Err("That screenshot is no longer open in the editor".to_owned()),
    }
  };
  let composer = app.clone();
  let composed = tauri::async_runtime::spawn_blocking(move || {
    compose_screenshot_workspace(&composer, &items, &screenshot_output)
  })
  .await
  .map_err(|error| error.to_string())??;
  super::super::project_thumbnail::edited::keep_composed_preview(&app, project, composed).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn layers_are_kept_by_their_place_and_renamed_on_reopening() {
    let window = json!({
      "backgroundColor": "#fff",
      "items": [
        { "id": 9, "output": { "radiusPercent": 2 } },
        { "id": 7, "output": { "radiusPercent": 1 } },
      ],
    });

    let kept = stored(window, &[7, 9]);
    assert_eq!(kept["layers"][0]["radiusPercent"], 1);
    assert_eq!(kept["layers"][1]["radiusPercent"], 2);
    assert_eq!(kept["canvas"]["backgroundColor"], "#fff");
    assert!(kept["canvas"].get("items").is_none());

    // Reopened with a third layer taken since: the first two keep their
    // settings under this session's ids, and the new one has none yet.
    let reopened = restored(kept, &[21, 22, 23]).unwrap();
    assert_eq!(
      reopened["items"],
      json!([
        { "id": 21, "output": { "radiusPercent": 1 } },
        { "id": 22, "output": { "radiusPercent": 2 } },
      ])
    );
  }
}
