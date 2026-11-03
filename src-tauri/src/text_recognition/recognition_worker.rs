// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use screenwide_imaging::CapturedImage;

use super::*;

const DECODE_ERROR: &str = "QR-like code could not be decoded.";

/// Reads `image`'s text and QR codes at once, as neither needs the other: the
/// QR codes on a thread of their own, the text where it always was read, so
/// the wait is the slower of the two rather than both. Both threads say
/// someone waits on them, as threads Rust spawns start at the default class.
pub(super) async fn recognize(
  image: Arc<CapturedImage>,
) -> Result<(Vec<RecognizedLine>, Vec<RecognizedQrCode>), String> {
  tauri::async_runtime::spawn_blocking(move || {
    let _urgency = crate::user_work::ThreadUrgency::waited();
    let CapturedImage {
      ref rgba,
      width,
      height,
    } = *image;
    std::thread::scope(|scope| {
      let qr_codes = scope.spawn(|| {
        let _urgency = crate::user_work::ThreadUrgency::waited();
        read_qr_codes(rgba, width, height)
      });
      let lines = read_text(rgba, width, height);
      let qr_codes = qr_codes
        .join()
        .map_err(|_| "Reading the QR codes failed".to_owned())?;
      lines.map(|lines| (lines, qr_codes))
    })
  })
  .await
  .map_err(|error| error.to_string())?
}

fn read_qr_codes(rgba: &[u8], width: u32, height: u32) -> Vec<RecognizedQrCode> {
  screenwide_qr::read(rgba, width, height)
    .into_iter()
    .map(|code| RecognizedQrCode {
      bounds: TextRect {
        x: code.bounds.x,
        y: code.bounds.y,
        width: code.bounds.width,
        height: code.bounds.height,
      },
      decode_error: code.content.is_none().then(|| DECODE_ERROR.to_owned()),
      content: code.content.unwrap_or_default(),
    })
    .collect()
}

#[cfg(target_os = "macos")]
fn read_text(rgba: &[u8], width: u32, height: u32) -> Result<Vec<RecognizedLine>, String> {
  macos::platform::recognize(rgba, width, height)
}

#[cfg(target_os = "windows")]
fn read_text(rgba: &[u8], width: u32, height: u32) -> Result<Vec<RecognizedLine>, String> {
  windows::platform::recognize(rgba, width, height)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn read_text(_: &[u8], _: u32, _: u32) -> Result<Vec<RecognizedLine>, String> {
  Err("Text recognition is not available on this platform".to_owned())
}
