// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn workspace_topology(
  panes: &[PreviewSurfacePane],
  bake_camera: bool,
) -> RecordingWorkspaceTopology {
  let mut pane_indices = panes.iter().map(|pane| pane.index).collect::<Vec<_>>();
  pane_indices.sort_unstable();
  pane_indices.dedup();
  RecordingWorkspaceTopology {
    bake_camera,
    pane_indices,
  }
}

pub(super) fn clear_inactive_pane_targets(targets: &mut [(u32, u32)], active: &[usize]) {
  for (index, target) in targets.iter_mut().enumerate() {
    if !active.contains(&index) {
      *target = (0, 0);
    }
  }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn recording_workspace_geometry(
  panes: &[PreviewSurfacePane],
  output: &RecordingOutputSettings,
) -> (PreviewSurfaceRect, (u32, u32)) {
  let left = panes
    .iter()
    .map(|pane| pane.rect.x)
    .fold(f64::INFINITY, f64::min);
  let top = panes
    .iter()
    .map(|pane| pane.rect.y)
    .fold(f64::INFINITY, f64::min);
  let right = panes
    .iter()
    .map(|pane| pane.rect.x + pane.rect.width)
    .fold(f64::NEG_INFINITY, f64::max);
  let bottom = panes
    .iter()
    .map(|pane| pane.rect.y + pane.rect.height)
    .fold(f64::NEG_INFINITY, f64::max);
  let bounds = PreviewSurfaceRect {
    x: left,
    y: top,
    width: (right - left).max(1.0),
    height: (bottom - top).max(1.0),
  };
  let fit = panes
    .iter()
    .find_map(|pane| {
      let width = if pane.index == 0 {
        output.primary.width
      } else {
        output.camera.width
      };
      (width > 0 && pane.rect.width > 0.0).then_some(pane.rect.width / f64::from(width))
    })
    .unwrap_or(1.0)
    .max(f64::EPSILON);
  (
    bounds,
    (
      (bounds.width / fit).round().max(1.0) as u32,
      (bounds.height / fit).round().max(1.0) as u32,
    ),
  )
}
