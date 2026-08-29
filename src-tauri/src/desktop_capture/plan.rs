// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{LogicalPosition, LogicalSize};

use crate::{capture_geometry::physical_capture_rect, recording::Region};

use super::{CapturePiece, CapturePlan, DesktopDisplay, DesktopRect, OutputLimits, PixelRect};

pub fn plan(
  displays: &[DesktopDisplay],
  anchor_id: u32,
  region: Region,
  limits: OutputLimits,
) -> Result<CapturePlan, String> {
  let anchor = displays
    .iter()
    .find(|display| display.id == anchor_id)
    .ok_or_else(|| "The Region anchor monitor is no longer available".to_owned())?;
  if !valid_region(region) || !displays.iter().all(|display| display.valid()) || !limits.valid() {
    return Err("The desktop Region geometry or output limits are invalid".to_owned());
  }
  let desktop_region = DesktopRect {
    x: anchor.x + region.position.x,
    y: anchor.y + region.position.y,
    width: region.size.width,
    height: region.size.height,
  };
  let intersections = intersections(displays, desktop_region);
  let natural_scale = intersections
    .iter()
    .map(|(display, _)| display.scale)
    .fold(0.0, f64::max);
  if intersections.is_empty() || natural_scale <= 0.0 {
    return Err("The selected Region does not intersect a display".to_owned());
  }
  let output_scale = constrained_scale(desktop_region, natural_scale, limits);
  let width = aligned_dimension(
    desktop_region.width,
    output_scale,
    limits.max_width,
    limits.alignment,
  );
  let height = aligned_dimension(
    desktop_region.height,
    output_scale,
    limits.max_height,
    limits.alignment,
  );
  if width == 0 || height == 0 {
    return Err("The selected Region is too small for the recording output".to_owned());
  }
  let pieces = intersections
    .into_iter()
    .filter_map(|(display, intersection)| {
      piece(
        display,
        intersection,
        desktop_region,
        output_scale,
        width,
        height,
      )
    })
    .collect::<Vec<_>>();
  if pieces.is_empty() {
    return Err("The selected Region has no visible output pixels".to_owned());
  }
  Ok(CapturePlan {
    desktop_region,
    width,
    height,
    output_scale,
    pieces,
  })
}

fn intersections(
  displays: &[DesktopDisplay],
  region: DesktopRect,
) -> Vec<(DesktopDisplay, DesktopRect)> {
  displays
    .iter()
    .filter_map(|display| {
      let intersection = DesktopRect::from_edges(
        region.x.max(display.x),
        region.y.max(display.y),
        region.right().min(display.x + display.width),
        region.bottom().min(display.y + display.height),
      );
      intersection.valid().then_some((*display, intersection))
    })
    .collect()
}

fn piece(
  display: DesktopDisplay,
  intersection: DesktopRect,
  region: DesktopRect,
  output_scale: f64,
  output_width: u32,
  output_height: u32,
) -> Option<CapturePiece> {
  let source = Region {
    position: LogicalPosition::new(intersection.x - display.x, intersection.y - display.y),
    size: LogicalSize::new(intersection.width, intersection.height),
  };
  let source_pixels = physical_capture_rect(
    source,
    display.scale,
    pixel_edge(display.width, display.scale),
    pixel_edge(display.height, display.scale),
  )?;
  let destination = PixelRect::from_edges(
    pixel_edge(intersection.x - region.x, output_scale).min(output_width),
    pixel_edge(intersection.y - region.y, output_scale).min(output_height),
    pixel_edge(intersection.right() - region.x, output_scale).min(output_width),
    pixel_edge(intersection.bottom() - region.y, output_scale).min(output_height),
  )?;
  Some(CapturePiece {
    display_id: display.id,
    source_pixels: PixelRect {
      x: source_pixels.x,
      y: source_pixels.y,
      width: source_pixels.width,
      height: source_pixels.height,
    },
    destination,
  })
}

impl DesktopDisplay {
  fn valid(self) -> bool {
    [self.x, self.y, self.width, self.height, self.scale]
      .iter()
      .all(|value| value.is_finite())
      && self.width > 0.0
      && self.height > 0.0
      && self.scale > 0.0
  }
}

impl DesktopRect {
  fn from_edges(left: f64, top: f64, right: f64, bottom: f64) -> Self {
    Self {
      x: left,
      y: top,
      width: right - left,
      height: bottom - top,
    }
  }

  fn valid(self) -> bool {
    self.width > 0.0 && self.height > 0.0
  }

  fn right(self) -> f64 {
    self.x + self.width
  }

  fn bottom(self) -> f64 {
    self.y + self.height
  }
}

impl PixelRect {
  fn from_edges(left: u32, top: u32, right: u32, bottom: u32) -> Option<Self> {
    (right > left && bottom > top).then_some(Self {
      x: left,
      y: top,
      width: right - left,
      height: bottom - top,
    })
  }
}

impl OutputLimits {
  fn valid(self) -> bool {
    self.max_width > 0 && self.max_height > 0 && self.max_pixels > 0 && self.alignment > 0
  }
}

fn constrained_scale(region: DesktopRect, natural: f64, limits: OutputLimits) -> f64 {
  let width_scale = f64::from(limits.max_width) / region.width;
  let height_scale = f64::from(limits.max_height) / region.height;
  let area_scale = (limits.max_pixels as f64 / (region.width * region.height)).sqrt();
  natural.min(width_scale).min(height_scale).min(area_scale)
}

fn aligned_dimension(points: f64, scale: f64, maximum: u32, alignment: u32) -> u32 {
  let pixels = pixel_edge(points, scale).min(maximum);
  pixels - pixels % alignment
}

fn valid_region(region: Region) -> bool {
  [
    region.position.x,
    region.position.y,
    region.size.width,
    region.size.height,
  ]
  .iter()
  .all(|value| value.is_finite())
    && region.size.width > 0.0
    && region.size.height > 0.0
}

fn pixel_edge(value: f64, scale: f64) -> u32 {
  (value * scale).round().max(0.0) as u32
}
