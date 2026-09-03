// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::gesture_after_input_update;

#[test]
fn a_redundant_enabled_update_preserves_the_active_pointer_gesture() {
  assert!(gesture_after_input_update(true, true));
  assert!(!gesture_after_input_update(true, false));
  assert!(!gesture_after_input_update(false, true));
}
