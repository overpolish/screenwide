// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral planning and timing for capture regions that span displays.
//!
//! Native adapters only provide display geometry and timestamped surfaces. This
//! module owns coordinate conversion, output sizing and latest-frame selection
//! so ScreenCaptureKit/Metal and Windows Graphics Capture/Direct3D cannot
//! quietly implement different recording behaviour.

mod plan;
mod timing;

pub use plan::plan;
#[cfg(test)]
use timing::CompositionTick;
pub use timing::FrameSynchronizer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopDisplay {
  pub id: u32,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub scale: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct CapturePiece {
  pub display_id: u32,
  /// Display-local device pixels, suitable for GPU texture crops on either OS.
  pub source_pixels: PixelRect,
  /// Pixels in the final composed canvas.
  pub destination: PixelRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelRect {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopRect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputLimits {
  pub max_width: u32,
  pub max_height: u32,
  pub max_pixels: u64,
  /// One for stills, two for video encoders with chroma-aligned dimensions.
  pub alignment: u32,
}

impl OutputLimits {
  pub const UNBOUNDED: Self = Self {
    max_width: u32::MAX,
    max_height: u32::MAX,
    max_pixels: u64::MAX,
    alignment: 1,
  };

  /// Cross-platform video contract: preserve panoramic geometry, cap total
  /// work to a 4K pixel budget, and never exceed common 8K encoder dimensions.
  pub const VIDEO: Self = Self {
    max_width: 7680,
    max_height: 4320,
    max_pixels: 3840 * 2160,
    alignment: 2,
  };
}

#[derive(Clone, Debug)]
pub struct CapturePlan {
  pub desktop_region: DesktopRect,
  pub width: u32,
  pub height: u32,
  pub output_scale: f64,
  pub pieces: Vec<CapturePiece>,
}

#[cfg(test)]
mod tests;
