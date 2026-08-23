// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::image_analysis::{detect_background_where, ImageRegion, ImageView};
use crate::ruler::analysis::{compute_gradients, detect_boxes, ComponentBox};
use crate::screenshots::NormalizedSourceRect;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecenterAnalysis {
  pub(super) background_color: String,
  pub(super) bounds: Option<ComponentBox>,
}

/// The smallest rectangle containing every detected UI component. Empty
/// space outside this union is the padding Recenter balances.
pub(super) fn analyse(
  rgba: &[u8],
  width: u32,
  height: u32,
  source_crop: NormalizedSourceRect,
  threshold: u8,
) -> Option<RecenterAnalysis> {
  if !source_crop.is_valid() {
    return None;
  }
  let left = (source_crop.x * f64::from(width)).floor() as u32;
  let top = (source_crop.y * f64::from(height)).floor() as u32;
  let right = ((source_crop.x + source_crop.width) * f64::from(width)).ceil() as u32;
  let bottom = ((source_crop.y + source_crop.height) * f64::from(height)).ceil() as u32;
  let region = ImageRegion {
    height: bottom.min(height).saturating_sub(top),
    width: right.min(width).saturating_sub(left),
    x: left,
    y: top,
  };
  let expected_length = width.checked_mul(height)?.checked_mul(4)? as usize;
  if region.width == 0 || region.height == 0 || rgba.len() < expected_length {
    return None;
  }
  let mut cropped =
    Vec::with_capacity(region.width.checked_mul(region.height)?.checked_mul(4)? as usize);
  for y in region.y..region.y + region.height {
    let start = ((y * width + region.x) * 4) as usize;
    let end = start + region.width as usize * 4;
    cropped.extend_from_slice(&rgba[start..end]);
  }
  let mut analysis = analyse_cropped(&cropped, region.width, region.height, threshold)?;
  if let Some(bounds) = analysis.bounds.as_mut() {
    bounds.x += region.x;
    bounds.y += region.y;
  }
  Some(analysis)
}

fn analyse_cropped(
  rgba: &[u8],
  width: u32,
  height: u32,
  threshold: u8,
) -> Option<RecenterAnalysis> {
  let boxes = detect_boxes(&compute_gradients(rgba, width, height), threshold);
  let bounds = boxes.first().map(|first| {
    let (mut left, mut top) = (first.x, first.y);
    let (mut right, mut bottom) = (first.x + first.width, first.y + first.height);
    for bounds in &boxes[1..] {
      left = left.min(bounds.x);
      top = top.min(bounds.y);
      right = right.max(bounds.x + bounds.width);
      bottom = bottom.max(bounds.y + bounds.height);
    }
    ComponentBox {
      x: left,
      y: top,
      width: right.saturating_sub(left),
      height: bottom.saturating_sub(top),
    }
  });
  let view = ImageView {
    height,
    rgba,
    width,
  };
  let region = ImageRegion {
    height,
    width,
    x: 0,
    y: 0,
  };
  let step = width.min(height).div_ceil(256).max(1);
  let sample = detect_background_where(view, region, step, |x, y| {
    bounds.is_none_or(|content| {
      x < content.x
        || x >= content.x + content.width
        || y < content.y
        || y >= content.y + content.height
    })
  })
  .or_else(|| detect_background_where(view, region, step, |_, _| true))?;
  Some(RecenterAnalysis {
    background_color: format!(
      "#{:02x}{:02x}{:02x}",
      sample.colour[0], sample.colour[1], sample.colour[2]
    ),
    bounds,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn full_crop() -> NormalizedSourceRect {
    NormalizedSourceRect {
      height: 1.0,
      width: 1.0,
      x: 0.0,
      y: 0.0,
    }
  }

  fn frame(rectangles: &[(u32, u32, u32, u32)]) -> Vec<u8> {
    let mut rgba = vec![30; 120 * 120 * 4];
    for pixel in rgba.chunks_exact_mut(4) {
      pixel[3] = 255;
    }
    for &(x, y, width, height) in rectangles {
      for row in y..y + height {
        for column in x..x + width {
          let index = ((row * 120 + column) * 4) as usize;
          rgba[index..index + 3].fill(220);
        }
      }
    }
    rgba
  }

  #[test]
  fn unions_separate_elements() {
    let rgba = frame(&[(12, 20, 24, 30), (72, 58, 30, 22)]);
    let analysis = analyse(&rgba, 120, 120, full_crop(), 30).expect("analysis");
    let bounds = analysis.bounds.expect("content bounds");
    assert!(bounds.x.abs_diff(12) <= 1);
    assert!(bounds.y.abs_diff(20) <= 1);
    assert!((bounds.x + bounds.width).abs_diff(102) <= 1);
    assert!((bounds.y + bounds.height).abs_diff(80) <= 1);
    assert_eq!(analysis.background_color, "#1e1e1e");
  }

  #[test]
  fn flat_frames_have_no_content_bounds() {
    let analysis = analyse(&frame(&[]), 120, 120, full_crop(), 30).expect("analysis");
    assert!(analysis.bounds.is_none());
    assert_eq!(analysis.background_color, "#1e1e1e");
  }

  #[test]
  fn crop_controls_content_and_background_analysis() {
    let mut rgba = frame(&[]);
    for y in 0..120 {
      for x in 60..120 {
        let index = ((y * 120 + x) * 4) as usize;
        rgba[index..index + 3].fill(42);
      }
    }
    for y in 35..85 {
      for x in 75..105 {
        let index = ((y * 120 + x) * 4) as usize;
        rgba[index..index + 3].fill(220);
      }
    }

    let analysis = analyse(
      &rgba,
      120,
      120,
      NormalizedSourceRect {
        height: 1.0,
        width: 0.5,
        x: 0.5,
        y: 0.0,
      },
      30,
    )
    .expect("analysis");

    assert_eq!(analysis.background_color, "#2a2a2a");
    let bounds = analysis.bounds.expect("content bounds");
    assert!(bounds.x.abs_diff(75) <= 1);
    assert!(bounds.y.abs_diff(35) <= 1);
    assert!((bounds.x + bounds.width).abs_diff(105) <= 1);
    assert!((bounds.y + bounds.height).abs_diff(85) <= 1);
  }
}
