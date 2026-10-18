// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::should_show;

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
