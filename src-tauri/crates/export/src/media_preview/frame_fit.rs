// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a video export carries the canvas its look is measured on onto the
//! frame it writes. H.264 wants even sides, which a scaled canvas seldom
//! has on both axes at once: 2338 rows at 75% are 1753.5. Rounding each
//! side on its own and stretching the canvas to fit drifts features apart
//! towards the edges, so the canvas is drawn at one scale on both axes and
//! centred in the even frame, at most a pixel of it cropped or padded.

use screenwide_scenes::CameraOverlaySettings;

use super::{BakedVideoExportOptions, VideoExportOptions};

/// The frame a canvas is written into and where the canvas lies in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameFit {
  /// The frame's sides, both even.
  pub width: u32,
  pub height: u32,
  /// Frame pixels to a canvas pixel, on both axes.
  pub scale: f64,
  /// Where the canvas's top left corner lands in the frame, within half a
  /// pixel of it either way.
  pub x: f64,
  pub y: f64,
}

impl FrameFit {
  /// A `width` by `height` canvas written at `video`'s resolution.
  pub fn of(width: u32, height: u32, video: VideoExportOptions) -> Self {
    let scale_percent = f64::from(video.resolution_scale_percent);
    let source_percent = f64::from(video.source_scale_percent.max(1));
    // In the order FFmpeg evaluates `resolution_filter`'s expression, so a
    // recording it scales comes out at these very sides.
    let drawn = |side: u32| f64::from(side) * scale_percent / source_percent;
    let even = |drawn: f64| (((drawn / 2.0 - 0.5).ceil() * 2.0).max(2.0)) as u32;
    let (drawn_width, drawn_height) = (drawn(width), drawn(height));
    let (frame_width, frame_height) = (even(drawn_width), even(drawn_height));
    Self {
      width: frame_width,
      height: frame_height,
      scale: scale_percent / source_percent,
      x: (f64::from(frame_width) - drawn_width) / 2.0,
      y: (f64::from(frame_height) - drawn_height) / 2.0,
    }
  }

  /// A canvas written at its own size, its sides as they are.
  pub fn unscaled(width: u32, height: u32) -> Self {
    Self {
      width,
      height,
      scale: 1.0,
      x: 0.0,
      y: 0.0,
    }
  }

  /// The frame column a canvas column `x` lands on.
  pub fn x(self, x: f64) -> f64 {
    self.x + x * self.scale
  }

  /// The frame row a canvas row `y` lands on.
  pub fn y(self, y: f64) -> f64 {
    self.y + y * self.scale
  }

  /// The canvas's left, top, width and height in the frame.
  pub fn canvas(self) -> [f64; 4] {
    [
      self.x,
      self.y,
      f64::from(self.width) - 2.0 * self.x,
      f64::from(self.height) - 2.0 * self.y,
    ]
  }

  /// `overlay`, placed on the canvas, placed on the frame instead.
  pub fn overlay(self, overlay: CameraOverlaySettings) -> CameraOverlaySettings {
    CameraOverlaySettings {
      camera_x: self.x(overlay.camera_x),
      camera_y: self.y(overlay.camera_y),
      camera_width: overlay.camera_width * self.scale,
      frame_height: overlay.frame_height * self.scale,
      frame_width: overlay.frame_width * self.scale,
      frame_x: self.x(overlay.frame_x),
      frame_y: self.y(overlay.frame_y),
      ..overlay
    }
  }

  /// A camera baked over the canvas, baked over the frame instead, which
  /// it is then measured on at its own size.
  pub fn camera(self, options: BakedVideoExportOptions) -> BakedVideoExportOptions {
    BakedVideoExportOptions {
      overlay: self.overlay(options.overlay),
      screen_height: self.height,
      screen_width: self.width,
      video: VideoExportOptions {
        resolution_scale_percent: 100,
        source_scale_percent: 100,
        ..options.video
      },
      ..options
    }
  }
}

#[cfg(test)]
mod tests;
