// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::{cg, cv, sc};

use crate::{
  capture_kit::{desktop_layout, windows_to_exclude},
  desktop_capture::{self, OutputLimits},
  screenshots::{desktop as still_composition, CapturedImage},
};

pub(super) async fn capture(
  content: &sc::ShareableContent,
  anchor_id: u32,
  region: crate::recording::Region,
  include_own_windows: bool,
  show_cursor: bool,
) -> Result<CapturedImage, String> {
  let displays = desktop_layout()?;
  let plan = desktop_capture::plan(&displays, anchor_id, region, OutputLimits::UNBOUNDED)?;
  let capture_displays = content.displays();
  let mut captured = Vec::with_capacity(plan.pieces.len());
  for piece in plan.pieces.iter().copied() {
    let layout = displays
      .iter()
      .find(|display| display.id == piece.display_id)
      .ok_or_else(|| "A composed display disappeared".to_owned())?;
    let display = capture_displays
      .iter()
      .find(|display| display.display_id().0 == piece.display_id)
      .ok_or_else(|| "A composed display is no longer available".to_owned())?;
    let rect = piece.source_pixels;
    let mut config = sc::StreamCfg::new();
    config.set_shows_cursor(show_cursor);
    config.set_pixel_format(cv::PixelFormat::_32_BGRA);
    config.set_color_space_name(cg::color_space::names::srgb());
    config.set_src_rect(cidre::cg::Rect::new(
      f64::from(rect.x) / layout.scale,
      f64::from(rect.y) / layout.scale,
      f64::from(rect.width) / layout.scale,
      f64::from(rect.height) / layout.scale,
    ));
    config.set_width(rect.width as usize);
    config.set_height(rect.height as usize);
    let filter = sc::ContentFilter::with_display_excluding_windows(
      display,
      &windows_to_exclude(content, include_own_windows),
    );
    captured.push((piece, super::capture_filtered(&filter, &config).await?));
  }
  still_composition::compose(&plan, captured)
}
