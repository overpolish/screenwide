// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
    27 => "\u{2212}",
    28 => "8",
    29 => "0",
    30 => "]",
    31 => "O",
    32 => "U",
    33 => "[",
    34 => "I",
    35 => "P",
    36 => "Enter",
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
    54 | 55 => "Win",
    // The shared Keyboard maps Shift to its 12px ArrowBigUp icon on both
    // platforms; this compact glyph keeps the Windows cap from becoming a
    // text-only variant while retaining the existing Win/Ctrl labels.
    56 | 60 => "⇧",
    57 => "Caps Lock",
    58 | 61 => "Alt",
    59 | 62 => "Ctrl",
    63 => "fn",
    65 => ".",
    67 => "*",
    69 => "+",
    71 => "Num Lock",
    75 => "/",
    76 => "Enter",
    78 => "\u{2212}",
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
    117 => "Del",
    118 => "F4",
    119 => "End",
    120 => "F2",
    121 => "Page Down",
    122 => "F1",
    123 => "\u{2190}",
    124 => "\u{2192}",
    125 => "\u{2193}",
    126 => "\u{2191}",
    // Passthrough keys with no macOS position, 0x0200 | the virtual key.
    0x0213 => "Pause",
    0x022c => "PrtScn",
    0x025d => "Menu",
    0x0291 => "Scroll Lock",
    _ => return format!("Key {code}"),
  };
  label.to_owned()
}

pub(super) fn is_modifier_key(code: u16) -> bool {
  matches!(code, 54..=56 | 58..=63)
}

/// Version-one sidecars stored a modifier mask on the single recorded key.
/// Expanding it here keeps old recordings looking like the grouped shortcut.
pub(super) fn prepared_shortcut(overlay: &KeyboardOverlay) -> Vec<(u16, KeyboardKey)> {
  let mut prepared = Vec::with_capacity(MAX_KEYS);
  let count = (overlay.key_count as usize).min(MAX_KEYS);
  for state in overlay.keys.iter().take(count) {
    if count == 1 && !is_modifier_key(state.key_code) {
      for (bit, code) in [55_u16, 59, 58, 56, 63].into_iter().enumerate() {
        if state.modifier_mask & (1 << bit) != 0 && prepared.len() < MAX_KEYS {
          prepared.push((code, *state));
        }
      }
    }
    if prepared.len() < MAX_KEYS {
      prepared.push((state.key_code, *state));
    }
  }
  prepared
}
