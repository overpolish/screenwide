// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::{cg, cv, objc, sc};

use crate::screenshots::CapturedImage;

/// Ask ScreenCaptureKit to scale before copying pixels into Rust memory.
pub(crate) fn capture_monitor_thumbnail(
  monitor_id: u32,
  width: u32,
  height: u32,
) -> Result<CapturedImage, String> {
  if width == 0 || height == 0 || width > 320 || height > 320 {
    return Err("Invalid monitor thumbnail dimensions".to_owned());
  }
  objc::ar_pool(|| {
    tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .map_err(|error| error.to_string())?
      .block_on(async {
        let content = sc::ShareableContent::current()
          .await
          .map_err(|error| error.to_string())?;
        let displays = content.displays();
        let display = displays
          .iter()
          .find(|display| display.display_id().0 == monitor_id)
          .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
        let filter = sc::ContentFilter::with_display_excluding_windows(
          display,
          &crate::capture_kit::windows_to_exclude(&content, true),
        );
        let mut cfg = sc::StreamCfg::new();
        cfg.set_width(width as usize);
        cfg.set_height(height as usize);
        cfg.set_scales_to_fit(true);
        cfg.set_shows_cursor(false);
        cfg.set_pixel_format(cv::PixelFormat::_32_BGRA);
        cfg.set_color_space_name(cg::color_space::names::srgb());
        super::capture_filtered(&filter, &cfg).await
      })
  })
}
