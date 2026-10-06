// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a selected camera mode can still be opened, answered with the same
//! lookups a recording's camera makes when it starts. The bar's warning and
//! the start therefore cannot disagree about a camera.

use nokhwa::{
  query,
  utils::{ApiBackend, CameraInfo},
};
use serde::Serialize;

use super::camera_id;
use crate::fault::{self, Fault};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CameraModeStatus {
  Available,
  /// The camera itself no longer enumerates.
  Missing,
  /// The camera is there but no longer offers this size at this rate.
  ModeUnavailable,
}

/// `width` and `height` are evened as both platforms' recordings even them.
pub(crate) fn camera_mode_status(
  device_id: &str,
  width: u32,
  height: u32,
  fps: u32,
) -> CameraModeStatus {
  let camera = query(ApiBackend::Auto).ok().and_then(|cameras| {
    cameras
      .into_iter()
      .find(|camera| camera_id(camera) == device_id)
  });
  let Some(camera) = camera else {
    return CameraModeStatus::Missing;
  };
  if fault::active(Fault::CameraMode) || !mode_opens(&camera, width & !1, height & !1, fps.max(1)) {
    return CameraModeStatus::ModeUnavailable;
  }
  CameraModeStatus::Available
}

#[cfg(target_os = "macos")]
fn mode_opens(camera: &CameraInfo, width: u32, height: u32, fps: u32) -> bool {
  crate::camera_frame_rate::resolve_device(&camera_id(camera), &camera.human_name()).is_ok_and(
    |device| crate::camera_frame_rate::find_format(&device, width, height, fps).is_some(),
  )
}

#[cfg(not(target_os = "macos"))]
fn mode_opens(camera: &CameraInfo, width: u32, height: u32, fps: u32) -> bool {
  crate::camera_format::resolve_exact_camera_format(camera.index(), width, height, fps).is_ok()
}

/// The start's view of the same answer: what to tell the user when it is not
/// `Available`, so they turn the camera off or choose again.
pub(crate) fn require_camera_mode(
  device_id: &str,
  width: u32,
  height: u32,
  fps: u32,
) -> Result<(), String> {
  match camera_mode_status(device_id, width, height, fps) {
    CameraModeStatus::Available => Ok(()),
    CameraModeStatus::Missing => Err(
      "The selected camera is no longer connected. Choose another camera in the recording bar, or turn the camera off."
        .to_owned(),
    ),
    CameraModeStatus::ModeUnavailable => Err(format!(
      "The camera no longer offers {width} × {height} at {fps} fps. Choose another camera mode in the recording bar, or turn the camera off."
    )),
  }
}

#[tauri::command]
pub async fn get_camera_mode_status(
  device_id: String,
  width: u32,
  height: u32,
  fps: u32,
) -> Result<CameraModeStatus, String> {
  tauri::async_runtime::spawn_blocking(move || camera_mode_status(&device_id, width, height, fps))
    .await
    .map_err(|error| error.to_string())
}
