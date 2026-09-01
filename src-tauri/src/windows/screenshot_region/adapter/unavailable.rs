// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

use super::RegionSceneRequest;

pub(crate) fn acquire_quick_screenshot_cursor(_app: &AppHandle) -> Result<(), String> {
  Ok(())
}

pub(crate) fn release_quick_screenshot_cursor(_app: &AppHandle) -> Result<(), String> {
  Ok(())
}

/// Windows deliberately reports the absent compositor until its Win32/D3D
/// adapter is installed. The portable request is already the contract it must
/// consume; no workflow-specific fallback is allowed here.
pub(crate) fn apply_region_scene(
  _app: &AppHandle,
  _target: tauri::WebviewWindow,
  _request: RegionSceneRequest,
) -> Result<bool, String> {
  Ok(false)
}

pub(crate) fn set_desktop_presented(
  _window: &tauri::WebviewWindow,
  _presented: bool,
) -> tauri::Result<()> {
  Ok(())
}

pub(crate) fn prepare_for_region_restore(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

pub(crate) fn prepare_for_screenshot(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

pub(crate) fn set_frame_visible(
  _window: &tauri::WebviewWindow,
  _visible: bool,
) -> Result<bool, String> {
  Ok(false)
}

pub(crate) fn set_magnifier_source(
  _window: &tauri::WebviewWindow,
  _image: crate::screenshots::CapturedImage,
) -> Result<bool, String> {
  Ok(false)
}
