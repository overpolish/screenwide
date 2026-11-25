// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The screenshot editor's canvas and layers, saved in the project.
//!
//! The window names layers by ids that last one session, so the project keeps
//! them by their place in the stack instead: the canvas, then each layer's
//! settings in the order its pictures were taken.

use serde_json::{json, Value};

use super::formats::{with_layer_keys, with_looks};
use super::*;

/// The open screenshot's project and its layers' ids, back to front, if it is
/// `artifact_id`.
pub(super) fn open_project(
  app: &AppHandle,
  artifact_id: u64,
) -> Result<(PathBuf, Vec<u64>), String> {
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

/// The window's workspace as the project keeps it, and the formats with
/// the workspaces their shapes keep aside, and the layers they show in
/// only some formats, kept the same way.
fn stored(workspace: Value, formats: Option<Value>, ids: &[u64]) -> Value {
  let mut saved = stored_workspace(workspace, ids);
  if let Some(formats) = formats {
    let formats = with_layer_keys(formats, |key| {
      let id = key.parse::<u64>().ok()?;
      Some(ids.iter().position(|each| *each == id)?.to_string())
    });
    saved["formats"] = with_looks(formats, |look| Some(stored_workspace(look, ids)));
  }
  saved
}

/// Formats as the project keeps them (`stored`), as the window takes them.
fn restored_formats(formats: Value, ids: &[u64]) -> Value {
  let formats = with_layer_keys(formats, |key| {
    Some(ids.get(key.parse::<usize>().ok()?)?.to_string())
  });
  with_looks(formats, |look| restored(look, ids))
}

fn stored_workspace(mut workspace: Value, ids: &[u64]) -> Value {
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

/// What a screenshot project keeps for a window whose layers are `ids`.
pub(crate) struct RestoredWorkspace {
  pub(crate) workspace: Option<Value>,
  /// The shapes the screenshot is made in, if it is made for platforms.
  pub(crate) formats: Option<Value>,
  /// The layers of `ids` it no longer holds: deleted in the editor, which
  /// reopening must not bring back as though they had been taken since it
  /// was saved.
  pub(crate) deleted: Vec<u64>,
}

/// The workspace and formats saved in the screenshot project `project`,
/// for a window whose layers are `ids`.
pub(crate) fn restored_workspace(project: &Path, ids: &[u64]) -> RestoredWorkspace {
  let Some(saved) = super::super::project_look::saved_look(project) else {
    return RestoredWorkspace {
      workspace: None,
      formats: None,
      deleted: Vec::new(),
    };
  };
  RestoredWorkspace {
    deleted: deleted(&saved, ids),
    formats: saved
      .get("formats")
      .cloned()
      .map(|formats| restored_formats(formats, ids)),
    workspace: restored(saved, ids),
  }
}

/// The layers of `ids` whose place in `saved` is empty. A layer added since
/// it was saved has no place there at all, and is not among them.
fn deleted(saved: &Value, ids: &[u64]) -> Vec<u64> {
  let Some(layers) = saved.get("layers").and_then(Value::as_array) else {
    return Vec::new();
  };
  ids
    .iter()
    .zip(layers)
    .filter(|(_, output)| output.is_null())
    .map(|(id, _)| *id)
    .collect()
}

/// Saves the open screenshot's canvas and layers in its project. `edited`
/// is false for the save that only lays a newly opened screenshot out, so
/// it is not taken for a change the user made.
#[tauri::command]
pub async fn set_screenshot_project_workspace(
  app: AppHandle,
  artifact_id: u64,
  edited: bool,
  revision: u64,
  workspace: Value,
  formats: Option<Value>,
) -> Result<(), String> {
  let (project, ids) = open_project(&app, artifact_id)?;
  if edited {
    super::temporary::set_unsaved(&app, true);
  }
  // Off the async runtime: a background picture is copied in the first time
  // it is saved.
  tauri::async_runtime::spawn_blocking(move || {
    let look = stored(workspace, formats, &ids);
    super::super::project_look::save_look(&project, revision, look)
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Composes the open screenshot as the window shows it and keeps it as its
/// project's preview, smaller, and as its large still.
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
  let composed = tauri::async_runtime::spawn_blocking(move || {
    compose_screenshot_workspace(&items, &screenshot_output)
  })
  .await
  .map_err(|error| error.to_string())??;
  use super::super::project_thumbnail::edited::{keep_composed_preview, keep_large_still};
  keep_large_still(&project, composed.clone()).await?;
  keep_composed_preview(&app, project, composed).await
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

    let kept = stored(window, None, &[7, 9]);
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

  #[test]
  fn a_format_keeps_its_layers_by_their_place_too() {
    let window = json!({ "items": [{ "id": 7, "output": { "radiusPercent": 1 } }] });
    let formats = json!({
      "active": "tiktok-shape",
      "list": [
        { "id": "main", "look": { "items": [{ "id": 7, "output": { "radiusPercent": 4 } }] } },
        { "id": "tiktok-shape", "platforms": ["tiktok"] },
      ],
      "showIn": { "layers": { "7": { "formats": ["tiktok"] } } },
    });

    let kept = stored(window, Some(formats), &[7]);
    assert_eq!(
      kept["formats"]["list"][0]["look"]["layers"][0]["radiusPercent"],
      4
    );
    assert_eq!(
      kept["formats"]["showIn"]["layers"],
      json!({ "0": { "formats": ["tiktok"] } })
    );

    let reopened = restored_formats(kept["formats"].clone(), &[31]);
    assert_eq!(
      reopened["list"][0]["look"]["items"],
      json!([{ "id": 31, "output": { "radiusPercent": 4 } }])
    );
    assert!(reopened["list"][1].get("look").is_none());
    assert_eq!(
      reopened["showIn"]["layers"],
      json!({ "31": { "formats": ["tiktok"] } })
    );
  }

  #[test]
  fn a_deleted_layer_is_told_apart_from_one_taken_since() {
    // Three layers, the middle one deleted before the save.
    let window = json!({
      "items": [
        { "id": 7, "output": { "radiusPercent": 1 } },
        { "id": 9, "output": { "radiusPercent": 3 } },
      ],
    });
    let kept = stored(window, None, &[7, 8, 9]);

    // Reopened with a fourth taken since: only the deleted one is named.
    assert_eq!(deleted(&kept, &[21, 22, 23, 24]), vec![22]);
  }
}
