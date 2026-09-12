// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::video::{open_asset, NativeVideoReader};
use crate::screenshots::CapturedImage;

/// Analyse the decoder's RGB values directly. AssetImageGenerator followed by
/// a Core Graphics bitmap colour-converts the frame, shifting the detected
/// inset away from the source pixels used by the native compositor.
pub(crate) fn source_frame_image(
  path: &std::path::Path,
  position_ms: u64,
  duration_ms: u64,
) -> Result<CapturedImage, String> {
  let asset = open_asset(path)?;
  let tracks = tauri::async_runtime::block_on(
    asset.load_tracks_with_media_type(cidre::av::MediaType::video()),
  )
  .map_err(|error| error.to_string())?;
  let track = tracks
    .get(0)
    .map_err(|_| "The recording has no video track".to_owned())?;
  let size = track.natural_size();
  let position_ms = position_ms.min(duration_ms.saturating_sub(1));
  let mut reader = NativeVideoReader::open(
    &asset,
    size.width as u32,
    size.height as u32,
    position_ms.saturating_sub(100),
    duration_ms,
  )?;
  reader
    .frame_at(position_ms)?
    .ok_or_else(|| "AVFoundation returned no source frame".to_owned())
}
