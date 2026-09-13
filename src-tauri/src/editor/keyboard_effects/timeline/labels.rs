// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn shortcut_label(shortcut: &Shortcut, legacy_modifier_expansion: bool) -> String {
  let mut labels: Vec<String> = Vec::new();
  if legacy_modifier_expansion && shortcut.keys.len() == 1 {
    let key = &shortcut.keys[0];
    if !is_modifier_key(key.key_code) {
      for (bit, label) in [
        (1, "Command"),
        (2, "Control"),
        (4, "Option"),
        (8, "Shift"),
        (16, "fn"),
      ] {
        if key.modifier_mask & bit != 0 {
          labels.push(label.to_owned());
        }
      }
    }
  }
  labels.extend(shortcut.keys.iter().map(|key| key_label(key.key_code)));
  labels.join(" ")
}

pub(super) fn is_modifier_key(code: u16) -> bool {
  matches!(code, 54..=56 | 58..=63)
}

/// Labels for macOS-normalized virtual keycodes used by both capture platforms.
pub(super) fn key_label(code: u16) -> String {
  let label = match code {
    0 => "A",
    1 => "S",
    2 => "D",
    3 => "F",
    4 => "H",
    5 => "G",
    6 => "Z",
    7 => "X",
    8 => "C",
    9 => "V",
    11 => "B",
    12 => "Q",
    13 => "W",
    14 => "E",
    15 => "R",
    16 => "Y",
    17 => "T",
    18 => "1",
    19 => "2",
    20 => "3",
    21 => "4",
    22 => "6",
    23 => "5",
    24 => "=",
    25 => "9",
    26 => "7",
    27 => "−",
    28 => "8",
    29 => "0",
    30 => "]",
    31 => "O",
    32 => "U",
    33 => "[",
    34 => "I",
    35 => "P",
    36 | 76 => "Enter",
    37 => "L",
    38 => "J",
    39 => "'",
    40 => "K",
    41 => ";",
    42 => "\\",
    43 => ",",
    44 => "/",
    45 => "N",
    46 => "M",
    47 => ".",
    48 => "Tab",
    49 => "Space",
    50 => "`",
    51 => "Backspace",
    53 => "Esc",
    54 | 55 => "Command",
    56 | 60 => "Shift",
    57 => "Caps Lock",
    58 | 61 => "Option",
    59 | 62 => "Control",
    63 => "fn",
    65 => ".",
    67 => "*",
    69 => "+",
    71 => "Clear",
    75 => "/",
    78 => "−",
    81 => "=",
    82 => "0",
    83 => "1",
    84 => "2",
    85 => "3",
    86 => "4",
    87 => "5",
    88 => "6",
    89 => "7",
    91 => "8",
    92 => "9",
    96 => "F5",
    97 => "F6",
    98 => "F7",
    99 => "F3",
    100 => "F8",
    101 => "F9",
    103 => "F11",
    105 => "F13",
    106 => "F16",
    107 => "F14",
    109 => "F10",
    111 => "F12",
    113 => "F15",
    114 => "Insert",
    115 => "Home",
    116 => "Page Up",
    117 => "Delete",
    118 => "F4",
    119 => "End",
    120 => "F2",
    121 => "Page Down",
    122 => "F1",
    123 => "←",
    124 => "→",
    125 => "↓",
    126 => "↑",
    _ => return format!("Key {code}"),
  };
  label.to_owned()
}
