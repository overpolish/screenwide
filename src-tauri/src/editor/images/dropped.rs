// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A picture dropped on the workspace, on its way to becoming an image.
//!
//! Each platform's drop target hands over what the drag carried, in the form
//! that keeps the most of it, and where on the picture it landed. The
//! picture is kept here, off the thread the drop arrived on, and the editor
//! is told: it places the image through the same command a paste uses, so
//! a recording is paused first and the image is timed, dressed and
//! committed as one edit.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::store::import_file;
use crate::editor::annotations::image::ImageArt;
use crate::editor::annotations::Annotation;

/// What a drag carried, as the platform hands it over.
pub(crate) enum DroppedPicture {
  /// An image file, from Finder or Explorer.
  File(PathBuf),
  /// A file the dragging app wrote out for the drop, as browsers on macOS
  /// do for an image, into a folder of its own: kept, then removed with its
  /// folder, since nothing else will. Windows hands the same file over as
  /// data.
  #[cfg(target_os = "macos")]
  Written(PathBuf),
  /// The picture's own file data, in any format the `image` crate reads.
  Data(Vec<u8>),
}

/// Where a drop landed: the layer under it, and the point in that layer's
/// image-normalised space. The twin of `ImagePoint` in
/// `src/features/editor/images/image-api.ts`.
#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Serialize)]
pub(crate) struct ImagePoint {
  pub(crate) layer: u32,
  pub(crate) x: f64,
  pub(crate) y: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageDropEvent {
  workspace: &'static str,
  art: ImageArt,
  at: Option<ImagePoint>,
}

/// The editor event a kept drop is announced with.
pub(crate) const IMAGE_DROP_EVENT: &str = "editor://image-drop";

/// The ids of `annotations`, to tell the image a drop places from those
/// already there.
pub(crate) fn annotation_ids(annotations: &[Annotation]) -> Vec<String> {
  annotations
    .iter()
    .map(|annotation| annotation.id.clone())
    .collect()
}

/// The annotation among `after` whose id `before` did not have.
pub(crate) fn fresh_id(before: &[String], after: &[Annotation]) -> Option<String> {
  after
    .iter()
    .find(|annotation| !before.contains(&annotation.id))
    .map(|annotation| annotation.id.clone())
}

fn keep(picture: DroppedPicture) -> Result<ImageArt, String> {
  match picture {
    DroppedPicture::File(path) => import_file(&path),
    #[cfg(target_os = "macos")]
    DroppedPicture::Written(path) => {
      let kept = import_file(&path);
      let _ = std::fs::remove_file(&path);
      if let Some(folder) = path.parent() {
        let _ = std::fs::remove_dir(folder);
      }
      kept
    }
    DroppedPicture::Data(bytes) => super::store::import_bytes(&bytes),
  }
}

/// Keeps `picture` and tells the `workspace` editor to place it `at` the
/// point it landed on, or in the middle of the layer in hand where no picture
/// was laid out. Returns at once: decoding a large picture is no work for the
/// thread a drop arrives on.
pub(crate) fn deliver(
  app: &AppHandle,
  workspace: &'static str,
  picture: DroppedPicture,
  at: Option<ImagePoint>,
) {
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || match keep(picture) {
    Ok(art) => {
      let _ = app.emit(IMAGE_DROP_EVENT, ImageDropEvent { workspace, art, at });
    }
    Err(error) => eprintln!("Could not use the dropped picture: {error}"),
  });
}
