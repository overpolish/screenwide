// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Semantic roles used to order and reuse keyboard overlay slots.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::exports::keyboard_effects) enum VisualRole {
  Modifier,
  Primary,
}

impl VisualRole {
  pub(in crate::exports::keyboard_effects) fn order(self) -> u8 {
    match self {
      Self::Modifier => 0,
      Self::Primary => 1,
    }
  }

  pub(in crate::exports::keyboard_effects) fn same_slot_kind(self, other: Self) -> bool {
    matches!((self, other), (Self::Modifier, Self::Modifier)) || self == other
  }
}

pub(in crate::exports::keyboard_effects) fn role(key_code: u16) -> VisualRole {
  if matches!(key_code, 54 | 55 | 56 | 58 | 59 | 60 | 61 | 62 | 63) {
    VisualRole::Modifier
  } else {
    VisualRole::Primary
  }
}
