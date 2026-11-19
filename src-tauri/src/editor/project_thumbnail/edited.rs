// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The preview the editor composes for its open recording, edit and all,
//! kept in the project for the browser's card.

use image::imageops::FilterType;
use tauri::Emitter;

use super::*;

/// The project of the recording open as `artifact_id`, while it still is.
pub(super) fn open_project(app: &AppHandle, artifact_id: u64) -> Result<PathBuf, String> {
  let state = app.state::<EditorState>();
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  match artifact.as_ref() {
    Some(EditorArtifact::Recording { id, project, .. }) if *id == artifact_id => {
      Ok(project.clone())
    }
    _ => Err("That recording is no longer open in the editor".to_owned()),
  }
}

/// Keeps `composed`, at most `STILL_WIDTH` wide, as the large still of the
/// project whose manifest is `project`: the screenshot as edited, for what
/// shows it larger than its card does, such as a teleprompter's cue. Laid
/// over black where it is see-through, as a JPEG has no alpha.
pub(crate) async fn keep_large_still(
  project: &Path,
  composed: CapturedImage,
) -> Result<(), String> {
  let still = screenwide_project::still_path(project);
  tauri::async_runtime::spawn_blocking(move || {
    let shrunk = screenwide_export::still_size::shrunk(composed, screenwide_project::STILL_WIDTH)?;
    let (width, height) = (shrunk.width, shrunk.height);
    let picture = image::RgbaImage::from_raw(width, height, shrunk.rgba)
      .ok_or_else(|| "The composed frame is not the size it says".to_owned())?;
    // Its colour is premultiplied by its alpha, so dropping the alpha is
    // laying it over black.
    let rgb = image::DynamicImage::ImageRgba8(picture).into_rgb8();
    write_atomically(&still, true, |partial| {
      let file = std::fs::File::create(partial).map_err(|error| error.to_string())?;
      image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), 85)
        .encode(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        .map_err(|error| error.to_string())
    })
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Keeps `composed`, smaller, as the preview of the project whose manifest
/// is `project`, and tells the browser it has a new one.
pub(crate) async fn keep_composed_preview(
  app: &AppHandle,
  project: PathBuf,
  composed: CapturedImage,
) -> Result<(), String> {
  let preview = screenwide_project::preview_path(&project);
  tauri::async_runtime::spawn_blocking(move || {
    let image = image::RgbaImage::from_raw(composed.width, composed.height, composed.rgba)
      .ok_or_else(|| "The composed frame is not the size it says".to_owned())?;
    let width = PREVIEW_WIDTH.min(image.width()).max(1);
    let height = (u64::from(image.height()) * u64::from(width) / u64::from(image.width().max(1)))
      .max(1) as u32;
    let smaller = image::imageops::resize(&image, width, height, FilterType::Triangle);
    write_atomically(&preview, true, |partial| {
      smaller
        .save_with_format(partial, image::ImageFormat::Png)
        .map_err(|error| error.to_string())
    })
  })
  .await
  .map_err(|error| error.to_string())??;
  let _ = app.emit(screenwide_project::library::STILL_EVENT, &project);
  Ok(())
}
