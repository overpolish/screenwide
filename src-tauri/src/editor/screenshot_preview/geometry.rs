// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::ScreenshotWorkspaceOutputSettings;
use crate::screenshots::ScreenshotOutputSettings;

const MINIMUM_CANVAS_SIZE: f64 = 64.0;

pub(super) fn fit_workspace_to_items(
  snapshot: &ScreenshotWorkspaceOutputSettings,
  moved_index: usize,
  moved_output: &ScreenshotOutputSettings,
) -> ScreenshotWorkspaceOutputSettings {
  let width = f64::from(snapshot.canvas.width.max(1));
  let height = f64::from(snapshot.canvas.height.max(1));
  let mut next = snapshot.clone();
  if let Some(item) = next.items.get_mut(moved_index) {
    item.output = moved_output.clone();
  }
  let mut left = 0.0_f64;
  let mut top = 0.0_f64;
  let mut right = width;
  let mut bottom = height;
  for item in &next.items {
    let output = &item.output;
    left = left.min(output.crop_x.floor());
    top = top.min(output.crop_y.floor());
    right = right.max((output.crop_x + output.crop_width).ceil());
    bottom = bottom.max((output.crop_y + output.crop_height).ceil());
  }
  let next_width = (right - left).round().max(MINIMUM_CANVAS_SIZE);
  let next_height = (bottom - top).round().max(MINIMUM_CANVAS_SIZE);
  next.canvas.width = next_width as u32;
  next.canvas.height = next_height as u32;
  // Growing past the canvas's own top left corner moves the origin every
  // placement is measured from, so everything in it shifts by the same amount.
  for item in &mut next.items {
    let output = &mut item.output;
    output.width = next_width as u32;
    output.height = next_height as u32;
    output.crop_x -= left;
    output.crop_y -= top;
    output.image_x -= left;
    output.image_y -= top;
  }
  next
}
