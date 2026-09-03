// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Keyboard phases. macOS caught these with a global `NSEvent` monitor; the
/// Windows overlay is `WS_EX_NOACTIVATE` and never focused, so this mapping is
/// wired but currently unreachable (see the plan's Deviations).
///
/// `held` reports the phase to fire on key-up for the latching keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyCommand {
  pub phase: u32,
  /// Set for a key that latches until it is released.
  pub release: Option<u32>,
}

/// Virtual-key codes for the ruler shortcuts, translated from the macOS
/// keycodes (`+input.m:499-559`): 7=X, 48=Tab, 51/117=Delete, 8=C, 6=Z, 16=Y,
/// 17=T, 46=M, 18/19=1/2, 9/4=V/H, 15=R.
pub(crate) fn key_command(
  vk: u16,
  command: bool,
  shift: bool,
  repeat: bool,
  latched: bool,
) -> Option<KeyCommand> {
  const X: u16 = 0x58;
  const TAB: u16 = 0x09;
  const BACKSPACE: u16 = 0x08;
  const DELETE: u16 = 0x2E;
  const C: u16 = 0x43;
  const Z: u16 = 0x5A;
  const Y: u16 = 0x59;
  const T: u16 = 0x54;
  const M: u16 = 0x4D;
  const ONE: u16 = 0x31;
  const TWO: u16 = 0x32;
  const V: u16 = 0x56;
  const H: u16 = 0x48;
  const R: u16 = 0x52;

  let plain = |phase: u32| {
    Some(KeyCommand {
      phase,
      release: None,
    })
  };
  match (command, vk) {
    (false, X) => plain(13),
    (false, TAB) => plain(14),
    (false, BACKSPACE | DELETE) => plain(16),
    (true, C) => plain(17),
    (true, Z) => plain(if shift { 19 } else { 18 }),
    (true, Y) => plain(19),
    (false, T) if !repeat => plain(29),
    (false, M) if !repeat => plain(33),
    (false, ONE | TWO) if !repeat && !latched => Some(KeyCommand {
      phase: if vk == ONE { 20 } else { 21 },
      release: Some(22),
    }),
    (false, V | H) if !repeat && !latched => Some(KeyCommand {
      phase: if vk == V { 26 } else { 27 },
      release: Some(28),
    }),
    (false, R) if !repeat && !latched => Some(KeyCommand {
      phase: 31,
      release: Some(32),
    }),
    _ => None,
  }
}
