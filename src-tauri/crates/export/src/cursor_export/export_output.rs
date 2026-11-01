// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas a scaled export draws on. The look is measured at the output's
//! reference size; an export at another resolution carries it, and what it
//! placed on it, onto the smaller or larger canvas before drawing, at one
//! scale on both axes (see `FrameFit`).

use screenwide_output::ScreenshotOutputSettings;

use super::CursorExportRequest;
use crate::media_preview::{BakedVideoExportOptions, FrameFit, VideoExportOptions};

/// `request`'s output settings at the resolution its video options ask for.
pub(super) fn scaled(request: &CursorExportRequest<'_>) -> ScreenshotOutputSettings {
  scaled_to(request.output, request.video)
}

/// `settings` at the resolution `video` asks for.
pub(super) fn scaled_to(
  settings: &ScreenshotOutputSettings,
  video: VideoExportOptions,
) -> ScreenshotOutputSettings {
  let fit = FrameFit::of(settings.width, settings.height, video);
  ScreenshotOutputSettings {
    width: fit.width,
    height: fit.height,
    crop_x: fit.x(settings.crop_x),
    crop_y: fit.y(settings.crop_y),
    crop_width: settings.crop_width * fit.scale,
    crop_height: settings.crop_height * fit.scale,
    image_x: fit.x(settings.image_x),
    image_y: fit.y(settings.image_y),
    image_width: settings.image_width * fit.scale,
    ..settings.clone()
  }
}

/// The camera `request` bakes in, carried from the reference canvas it was
/// placed on onto the canvas the export draws, `request.output`.
pub(super) fn camera_on_canvas(
  request: &CursorExportRequest<'_>,
) -> Option<BakedVideoExportOptions> {
  request.camera.map(|(_, options)| {
    FrameFit::of(options.screen_width, options.screen_height, request.video).camera(options)
  })
}
