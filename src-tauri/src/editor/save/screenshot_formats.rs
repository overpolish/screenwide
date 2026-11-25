// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every format of a screenshot exported at once: each its own workspace,
//! laid out on its own canvas, composed and written in turn; a format that
//! comes out the same as the one before it is a copy of that file.

use super::*;

/// One format's file in an export of every format of a screenshot.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotFormatExport {
  pub file_stem: String,
  /// The workspace the format is published as, without what it leaves out.
  pub screenshot_output: ScreenshotWorkspaceOutputSettings,
  /// Written as a copy of the format before it, which comes out the same.
  #[serde(default)]
  pub is_copy: bool,
}

/// Exports every format of a screenshot at once, each in the image format
/// and colours `options` choose, in the order `formats` give them. The
/// finished island shows the last file.
#[tauri::command]
pub async fn save_screenshot_formats(
  app: AppHandle,
  window: tauri::WebviewWindow,
  options: RecordingExportOptions,
  formats: Vec<ScreenshotFormatExport>,
) -> Result<Option<super::export_command::ExportedFile>, String> {
  if formats.is_empty() {
    return Err("There is no format to export".to_owned());
  }
  let work: super::export_command::Work =
    Box::new(move |artifact, writing, _, cancelled, options| {
      let EditorArtifact::Screenshot { items, .. } = artifact else {
        return Err("Only a screenshot is exported in its formats".to_owned());
      };
      let mut written = Vec::new();
      let result = write_formats(
        items,
        (writing, cancelled),
        &options,
        &formats,
        &mut written,
      );
      // One cancelled or failed takes back every file the export made.
      if !matches!(result, Ok(Some(_))) {
        for path in &written {
          let _ = std::fs::remove_file(path);
        }
      }
      result
    });
  super::export_command::run_export(app, window, options, work).await
}

/// Writes `formats` into `writing`, each path pushed onto `written` as its
/// file is made; none where `cancelled` is set between two.
fn write_formats(
  items: &[ScreenshotItem],
  (writing, cancelled): (&Path, &AtomicBool),
  options: &RecordingExportOptions,
  formats: &[ScreenshotFormatExport],
  written: &mut Vec<PathBuf>,
) -> Result<Option<Vec<PathBuf>>, String> {
  let extension = options.screenshot_format.extension();
  for format in formats {
    if cancelled.load(Ordering::Relaxed) {
      return Ok(None);
    }
    let stem = sanitize_file_stem(&format.file_stem)
      .ok_or_else(|| "That file name cannot be used".to_owned())?;
    let path = unique_path(writing, &stem, extension, &|candidate| candidate.exists());
    match written.last().filter(|_| format.is_copy) {
      Some(before) => {
        std::fs::copy(before, &path)
          .map_err(|error| format!("Could not copy the export: {error}"))?;
      }
      None => {
        let composed = compose_screenshot_workspace(items, &format.screenshot_output)?;
        options
          .screenshot_format
          .write(&composed, &path, options.screenshot_full_colour)?;
      }
    }
    written.push(path);
  }
  Ok(Some(written.clone()))
}
