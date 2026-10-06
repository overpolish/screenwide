// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot workspace kept as a project.
//!
//! The first capture makes the project; each one taken while the editor has
//! it open is added to it as another layer, until the editor lets it go.
//! Each layer's picture is written to `media/` as it arrives, losslessly, and
//! the canvas and layers are saved in the project's look as they change, so
//! the project reopens as it was left. A capture copied to the clipboard
//! alone never reaches the editor, and makes no project.

pub(crate) mod workspace;

use super::artifact::next_id;
use super::artifact_present::{present_new, Origin};
use super::*;
use crate::project::{ScreenshotLayer, ScreenshotManifest};

pub(crate) use workspace::restored_workspace;

/// Hands a freshly captured still to the editor window, seeding its layer
/// with the live annotations the still covered. `scale_factor` is the
/// display scale it was captured at. The still is kept in the open
/// screenshot's project as another layer, or in a new project when the
/// editor has none.
pub fn present_screenshot(
  app: &AppHandle,
  image: CapturedImage,
  scale_factor: f64,
  annotations: Vec<Annotation>,
  suggested_file_stem: String,
) -> Result<(), String> {
  let item = ScreenshotItem {
    annotations,
    id: next_id(app),
    image,
    scale_factor,
  };
  let state = app.state::<EditorState>();
  let open_project = match state
    .screenshot
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
  {
    Some(EditorArtifact::Screenshot { project, .. }) => Some(project.clone()),
    _ => None,
  };
  let project = keep_layer(app, open_project.as_deref(), &suggested_file_stem, &item)?;

  if open_project.is_some() {
    let mut artifact = state
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(EditorArtifact::Screenshot { items, .. }) = artifact
      .as_mut()
      .filter(|artifact| matches!(artifact, EditorArtifact::Screenshot { project: open, .. } if *open == project))
    else {
      // The editor let the project go while the picture was being kept: it
      // is opened again, now holding this layer too.
      drop(artifact);
      *state
        .capture_reservation
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
      return open(app, &project);
    };
    items.push(item);
    *state
      .capture_reservation
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    drop(artifact);
    emit_snapshot(app, EditorKind::Screenshot);
    window::show(app, EditorKind::Screenshot).map_err(|error| error.to_string())?;
    let _ = crate::app_windows::hide_recording_ui(app.clone());
    return Ok(());
  }

  present_new(
    app,
    EditorArtifact::Screenshot {
      id: next_id(app),
      items: vec![item],
      project,
      suggested_file_stem,
    },
    Origin::Capture,
  )
}

/// Writes `bytes` as `name` in the project, through a neighbouring file so a
/// reader never sees half a picture.
fn write_picture(project: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
  let target = crate::project::resolve(project, name)?;
  let partial = target.with_extension("png.part");
  std::fs::write(&partial, bytes)
    .and_then(|()| std::fs::rename(&partial, &target))
    .map_err(|error| {
      let _ = std::fs::remove_file(&partial);
      format!("The screenshot could not be kept: {error}")
    })
}

fn layer(item: &ScreenshotItem, image: String) -> ScreenshotLayer {
  ScreenshotLayer {
    annotations: item
      .annotations
      .iter()
      .filter_map(|annotation| serde_json::to_value(annotation).ok())
      .collect(),
    image,
    scale_factor: item.scale_factor,
  }
}

/// Keeps `item` as a new layer of `project`, or as the first of a new
/// project called `title` when there is none, and says which project.
pub(super) fn keep_layer(
  app: &AppHandle,
  project: Option<&Path>,
  title: &str,
  item: &ScreenshotItem,
) -> Result<PathBuf, String> {
  let png = crate::screenshots::encoding::encode_truecolor_png(&item.image)?;
  if let Some(file) = project {
    let mut written = Ok(());
    crate::project::update(file, |manifest| {
      let Some(screenshot) = manifest.screenshot.as_mut() else {
        written = Err("This project is not a screenshot".to_owned());
        return false;
      };
      let name = format!("media/layer-{}.png", screenshot.layers.len() + 1);
      written = write_picture(file, &name, &png);
      if written.is_ok() {
        screenshot.layers.push(layer(item, name));
      }
      written.is_ok()
    })?;
    return written.map(|()| file.to_path_buf());
  }
  let created = crate::project::create(app, title)?;
  let name = "media/layer-1.png".to_owned();
  let kept = write_picture(&created.file, &name, &png).and_then(|()| {
    crate::project::write(
      &created.file,
      &crate::project::Manifest::screenshot(ScreenshotManifest {
        layers: vec![layer(item, name)],
      }),
    )
  });
  if let Err(error) = kept {
    created.remove();
    return Err(error);
  }
  crate::project::library::remember(app, &created.file);
  Ok(created.file)
}

/// Opens the screenshot project whose manifest is `file` in the editor, in
/// place of whatever screenshot it holds: that one is saved, so nothing is
/// lost.
pub(super) fn open(app: &AppHandle, file: &Path) -> Result<(), String> {
  let open = matches!(
    app
      .state::<EditorState>()
      .screenshot
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .as_ref(),
    Some(EditorArtifact::Screenshot { project, .. }) if project == file
  );
  if open {
    super::workspace::focus_pending(app, EditorKind::Screenshot);
    return Ok(());
  }
  super::project_look::adopt_pictures(file);
  super::project_look::clean_pictures(file);
  let manifest = crate::project::read(file)?;
  let screenshot = manifest
    .screenshot
    .ok_or_else(|| "This project is not a screenshot".to_owned())?;
  // A layer's first annotations seed it until the editor has saved the
  // layer itself; after that they are part of what was saved.
  let seeds_layers = manifest.look.is_none();
  let items = screenshot
    .layers
    .iter()
    .map(|layer| {
      let image = image::open(crate::project::resolve(file, &layer.image)?)
        .map_err(|error| format!("A picture in this project could not be read: {error}"))?
        .into_rgba8();
      Ok(ScreenshotItem {
        annotations: if seeds_layers {
          layer
            .annotations
            .iter()
            .filter_map(|annotation| serde_json::from_value(annotation.clone()).ok())
            .collect()
        } else {
          Vec::new()
        },
        id: next_id(app),
        image: CapturedImage {
          width: image.width(),
          height: image.height(),
          rgba: image.into_raw(),
        },
        scale_factor: layer.scale_factor,
      })
    })
    .collect::<Result<Vec<_>, String>>()?;
  if items.is_empty() {
    return Err("This screenshot project has no pictures".to_owned());
  }
  crate::project::library::remember(app, file);
  present_new(
    app,
    EditorArtifact::Screenshot {
      id: next_id(app),
      items,
      project: file.to_path_buf(),
      suggested_file_stem: file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned(),
    },
    Origin::Project,
  )
}
