// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{clear_inactive_pane_targets, workspace_topology, PreviewSurfacePane};
use crate::editor::preview_platform::PreviewSurfaceRect;

#[test]
fn reenabled_panes_require_a_fresh_present() {
  let mut targets = [(3_840, 2_160), (1_920, 1_080)];
  clear_inactive_pane_targets(&mut targets, &[0]);
  assert_eq!(targets, [(3_840, 2_160), (0, 0)]);

  let camera_needs_present = targets[1] == (0, 0);
  assert!(camera_needs_present);
}

#[test]
fn workspace_topology_distinguishes_camera_modes_and_active_panes() {
  let pane = |index| PreviewSurfacePane {
    index,
    rect: PreviewSurfaceRect {
      height: 100.0,
      width: 100.0,
      x: 0.0,
      y: 0.0,
    },
  };
  let primary = workspace_topology(&[pane(0)], false);
  let split = workspace_topology(&[pane(1), pane(0)], false);
  let baked = workspace_topology(&[pane(0)], true);

  assert_ne!(primary, split);
  assert_ne!(primary, baked);
  assert_eq!(split, workspace_topology(&[pane(0), pane(1)], false));
}
