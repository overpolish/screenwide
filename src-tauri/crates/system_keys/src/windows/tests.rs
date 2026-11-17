// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn only_the_exact_combinations_are_taken() {
  let win_shift = Modifiers {
    shift: true,
    win: true,
    ..Modifiers::default()
  };
  assert_eq!(
    matched(u32::from(VK_S.0), win_shift),
    Some(SystemKey::ScreenClip)
  );
  assert_eq!(
    matched(
      u32::from(VK_S.0),
      Modifiers {
        control: true,
        ..win_shift
      }
    ),
    None
  );
  assert_eq!(
    matched(u32::from(VK_SNAPSHOT.0), Modifiers::default()),
    Some(SystemKey::PrintScreen)
  );
  // Alt+Print Screen still copies the window in front.
  assert_eq!(
    matched(
      u32::from(VK_SNAPSHOT.0),
      Modifiers {
        alt: true,
        ..Modifiers::default()
      }
    ),
    None
  );
}
