// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! What a reference's window asks of it: its layout round the island, its
//! picture's opacity, and its island's actions.

use chrono::Local;
use serde::Serialize;
use tauri::{image::Image, AppHandle, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use ts_rs::TS;

use super::{engine, read_png, references, Reference};
use screenwide_osc::geometry::Size;

fn with_reference<T>(
  window: &WebviewWindow,
  read: impl FnOnce(&Reference) -> T,
) -> Result<T, String> {
  references()
    .as_ref()
    .and_then(|references| references.get(window.label()))
    .map(read)
    .ok_or_else(|| "This reference is no longer open".to_owned())
}

/// Where the island stands in a reference's window, in points from its top
/// left, and whether it hangs over the picture rather than under it.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReferenceLayout {
  pub island_x: f64,
  pub island_y: f64,
  pub island_above: bool,
}

/// Closes the reference, whose picture goes with its window.
#[tauri::command]
pub fn close_reference(window: WebviewWindow) -> Result<(), String> {
  window.close().map_err(|error| error.to_string())
}

/// Fades the picture out as the island tucks away, before it closes.
#[tauri::command]
pub fn fade_reference(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let label = window.label().to_owned();
  engine::on_main(&app, move || engine::fade_out(&label))
}

/// Whether Back to origin has somewhere to put the reference back: an area
/// it was taken from, which a dropped picture or a still of a whole screen
/// or window does not have.
#[tauri::command]
pub fn reference_returns_to_origin(window: WebviewWindow) -> Result<bool, String> {
  with_reference(&window, |reference| reference.placement.returns)
}

/// Puts the reference back over the area it was taken from, at its true
/// size.
#[tauri::command]
pub fn reference_to_origin(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let origin = with_reference(&window, |reference| {
    let placement = reference.placement;
    placement.returns.then_some(placement.origin).flatten()
  })?;
  let Some(origin) = origin else {
    return Ok(());
  };
  let label = window.label().to_owned();
  engine::on_main(&app, move || engine::to_origin(&label, origin))
}

/// Puts the reference back to its true size, its middle where it is: what a
/// reference with nowhere to return to has in place of Back to origin.
#[tauri::command]
pub fn reset_reference_size(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let label = window.label().to_owned();
  engine::on_main(&app, move || engine::reset_size(&label))
}

/// Fits the window round the picture and an island of `width` by `height`
/// points by it, gap included. Answers where the island stands.
#[tauri::command]
pub fn fit_reference(
  app: AppHandle,
  window: WebviewWindow,
  width: f64,
  height: f64,
) -> Result<ReferenceLayout, String> {
  let label = window.label().to_owned();
  engine::on_main(&app, move || {
    engine::fit_island(&label, Size { width, height })
  })?
  .ok_or_else(|| "This reference is no longer open".to_owned())
}

/// Shows the picture at `opacity`, from 0 to 1. Its chrome stays as it is.
#[tauri::command]
pub fn set_reference_opacity(
  app: AppHandle,
  window: WebviewWindow,
  opacity: f64,
) -> Result<(), String> {
  let label = window.label().to_owned();
  engine::on_main(&app, move || {
    engine::set_opacity(&label, opacity.clamp(0.0, 1.0));
  })
}

/// Copies the picture as the window shows it.
#[tauri::command]
pub async fn copy_reference(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let path = with_reference(&window, |reference| reference.shown.clone())?;
  let image = tauri::async_runtime::spawn_blocking(move || read_png(&path))
    .await
    .map_err(|error| error.to_string())??;
  app
    .clipboard()
    .write_image(&Image::new(&image.rgba, image.width, image.height))
    .map_err(|error| error.to_string())
}

/// Hands the still to the screenshot editor, with the annotations drawn over
/// it as a layer of their own, as a screenshot opens there. The reference
/// stays on screen.
#[tauri::command]
pub async fn open_reference_in_editor(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
  let (path, annotations, scale) = with_reference(&window, |reference| {
    (
      reference.original.clone(),
      reference.annotations.clone(),
      reference.scale,
    )
  })?;
  crate::editor::reserve_screenshot_workspace(&app)?;
  let opened = tauri::async_runtime::spawn_blocking(move || read_png(&path))
    .await
    .map_err(|error| error.to_string())
    .and_then(|read| read)
    .and_then(|image| {
      crate::editor::present_screenshot(
        &app,
        image,
        scale,
        annotations,
        crate::screenshots::capture_file_stem(Local::now().naive_local()),
      )
    });
  if opened.is_err() {
    crate::editor::release_screenshot_workspace(&app);
  }
  opened
}
