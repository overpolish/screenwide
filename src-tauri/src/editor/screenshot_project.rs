// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot workspace kept as a project.
//!
//! The first capture makes the project; each one taken while the editor has
//! it open is added to it as another layer, until the editor lets it go.
//! Each layer's picture is written to `media/` as it arrives, losslessly, and
//! the canvas and layers are saved in the project's look as they change, so
//! the project reopens as it was left. A capture opened in the editor is kept
//! in a temporary project until the editor's Save keeps it in Projects
//! (`temporary`); one copied to the clipboard is kept in Projects at once
//! only by its island's Save (`save_screenshot`).

pub(crate) mod blank;
pub(crate) mod temporary;
pub(crate) mod workspace;

use super::artifact::next_id;
use super::artifact_present::{present_new, Origin};
use super::project_open::OpenFailure;
use super::*;
use screenwide_project::{ScreenshotLayer, ScreenshotManifest};

pub(crate) use blank::open_blank_canvas;
pub(crate) use workspace::restored_workspace;

/// Hands a freshly captured still to the editor window, seeding its layer
/// with the live annotations the still covered. `scale_factor` is the
/// display scale it was captured at. The still is kept in the open
/// screenshot's project as another layer, or in a new temporary project when
/// the editor has none. `kept_elsewhere` says the still is on the clipboard
/// or on screen too, so closing it unchanged loses nothing.
pub fn present_screenshot(
  app: &AppHandle,
  image: CapturedImage,
  scale_factor: f64,
  annotations: Vec<Annotation>,
  suggested_file_stem: String,
  kept_elsewhere: bool,
) -> Result<(), String> {
  let item = ScreenshotItem {
    annotations,
    blank: false,
    id: next_id(app),
    image: Arc::new(image),
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
  let project = keep_layer(
    app,
    open_project.as_deref(),
    &suggested_file_stem,
    &item,
    open_project.is_none(),
  )?;

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
      super::workspace::settle(app, EditorKind::Screenshot);
      return open(app, &project).map_err(|failure| failure.into_reason(&project));
    };
    items.push(item);
    temporary::set_unsaved(app, true);
    super::workspace::settle(app, EditorKind::Screenshot);
    drop(artifact);
    emit_snapshot(app, EditorKind::Screenshot);
    window::show(app, EditorKind::Screenshot, |app| {
      let _ = crate::app_windows::hide_recording_ui(app.clone());
    });
    return Ok(());
  }

  temporary::set_unsaved(app, !kept_elsewhere);
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

/// Keeps a freshly captured still as a project of its own without opening
/// it, seeding its layer with the live annotations it covered, and says
/// where.
pub fn save_screenshot(
  app: &AppHandle,
  image: CapturedImage,
  scale_factor: f64,
  annotations: Vec<Annotation>,
  title: &str,
) -> Result<PathBuf, String> {
  let item = ScreenshotItem {
    annotations,
    blank: false,
    id: next_id(app),
    image: Arc::new(image),
    scale_factor,
  };
  keep_layer(app, None, title, &item, false)
}

/// Writes `image` as the PNG `name` in the project, through a neighbouring
/// file so a reader never sees half a picture.
fn write_picture(project: &Path, name: &str, image: &CapturedImage) -> Result<(), String> {
  let target = screenwide_project::resolve(project, name)?;
  let partial = target.with_extension("png.part");
  let written = std::fs::File::create(&partial)
    .map_err(|error| error.to_string())
    .and_then(|file| {
      let mut out = std::io::BufWriter::new(file);
      screenwide_output::encoding::write_truecolor_png(image, &mut out)?;
      std::io::Write::flush(&mut out).map_err(|error| error.to_string())
    })
    .and_then(|()| std::fs::rename(&partial, &target).map_err(|error| error.to_string()));
  written.map_err(|error| {
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
    blank: item.blank,
    image,
    scale_factor: item.scale_factor,
  }
}

/// Keeps `item` as a new layer of `project`, or as the first of a new
/// project called `title` when there is none, and says which project. A new
/// project is `temporary`, out of Projects, or kept in Projects.
pub(super) fn keep_layer(
  app: &AppHandle,
  project: Option<&Path>,
  title: &str,
  item: &ScreenshotItem,
  temporary: bool,
) -> Result<PathBuf, String> {
  if let Some(file) = project {
    let mut written = Ok(());
    screenwide_project::update(file, |manifest| {
      let Some(screenshot) = manifest.screenshot.as_mut() else {
        written = Err("This project is not a screenshot".to_owned());
        return false;
      };
      let name = format!("media/layer-{}.png", screenshot.layers.len() + 1);
      written = write_picture(file, &name, &item.image);
      if written.is_ok() {
        screenshot.layers.push(layer(item, name));
      }
      written.is_ok()
    })?;
    return written.map(|()| file.to_path_buf());
  }
  let created = if temporary {
    temporary::create(app, title)?
  } else {
    crate::projects_folder::create(app, title)?
  };
  let name = "media/layer-1.png".to_owned();
  let kept = write_picture(&created.file, &name, &item.image).and_then(|()| {
    screenwide_project::write(
      &created.file,
      &screenwide_project::Manifest::screenshot(ScreenshotManifest {
        layers: vec![layer(item, name)],
      }),
    )
  });
  if let Err(error) = kept {
    created.remove();
    return Err(error);
  }
  if !temporary {
    screenwide_project::library::remember(app, &created.file);
  }
  Ok(created.file)
}

/// Opens the screenshot project whose manifest is `file` in the editor, in
/// place of whatever screenshot it holds: that one is saved, so nothing is
/// lost.
pub(super) fn open(app: &AppHandle, file: &Path) -> Result<(), OpenFailure> {
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
  let manifest = screenwide_project::read(file)?;
  let screenshot = manifest
    .screenshot
    .ok_or_else(|| OpenFailure::damaged("This project is not a screenshot"))?;
  // A layer's first annotations seed it until the editor has saved the
  // layer itself; after that they are part of what was saved.
  let seeds_layers = manifest.look.is_none();
  let items = screenshot
    .layers
    .iter()
    .map(|layer| {
      let path = screenwide_project::resolve(file, &layer.image).map_err(OpenFailure::damaged)?;
      let image = screenwide_output::read_rgba(&path)
        .map_err(|error| OpenFailure::picture_unreadable(format!("{}: {error}", path.display())))?;
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
        image: Arc::new(image),
        scale_factor: layer.scale_factor,
        blank: layer.blank,
      })
    })
    .collect::<Result<Vec<_>, OpenFailure>>()?;
  if items.is_empty() {
    return Err(OpenFailure::damaged(
      "The screenshot project has no pictures",
    ));
  }
  screenwide_project::library::remember(app, file);
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
  .map_err(OpenFailure::from)
}
