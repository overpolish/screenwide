// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  opaque_snapshot_target, peer_pointer_style, should_show, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

#[test]
fn peer_passthrough_preserves_layering_and_every_unrelated_extended_style() {
  let base = (0x0012_3456_isize | WS_EX_LAYERED.0 as isize) & !(WS_EX_TRANSPARENT.0 as isize);
  let transparent = peer_pointer_style(base, true);
  assert_ne!(transparent & WS_EX_TRANSPARENT.0 as isize, 0);
  assert_ne!(transparent & WS_EX_LAYERED.0 as isize, 0);
  assert_eq!(peer_pointer_style(transparent, false), base);
}

#[test]
fn only_presented_peers_are_ordered_on_screen() {
  // The root follows the scene alone.
  assert!(should_show(true, true, false));
  assert!(!should_show(true, false, true));
  // A peer needs both the scene and the desktop presentation.
  assert!(should_show(false, true, true));
  assert!(!should_show(false, true, false));
  assert!(!should_show(false, false, true));
}

#[test]
fn every_presented_snapshot_uses_opaque_target_alpha() {
  assert!(opaque_snapshot_target(true, true));
  assert!(!opaque_snapshot_target(false, true));
  assert!(!opaque_snapshot_target(true, false));
}
