// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) async fn recognize(
  rgba: Vec<u8>,
  width: u32,
  height: u32,
) -> Result<(Vec<RecognizedLine>, Vec<RecognizedQrCode>), String> {
  tauri::async_runtime::spawn_blocking(move || {
    let qr_codes = qr::recognize(&rgba, width, height);
    #[cfg(target_os = "macos")]
    return platform_macos::recognize(&rgba, width, height).map(|lines| (lines, qr_codes));

    #[cfg(target_os = "windows")]
    return platform_windows::recognize(&rgba, width, height).map(|lines| (lines, qr_codes));

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Text recognition is not available on this platform".to_owned())
  })
  .await
  .map_err(|error| error.to_string())?
}
