// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// How faint an edge [`super::detect_boxes`] still counts, as the threshold it
/// is given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tolerance {
  ClearEdges,
  #[default]
  Balanced,
  SubtleEdges,
}

impl Tolerance {
  pub const fn threshold(self) -> u8 {
    match self {
      Self::ClearEdges => 40,
      Self::Balanced => 24,
      Self::SubtleEdges => 5,
    }
  }

  pub const fn next(self) -> Self {
    match self {
      Self::ClearEdges => Self::Balanced,
      Self::Balanced => Self::SubtleEdges,
      Self::SubtleEdges => Self::ClearEdges,
    }
  }

  pub const fn index(self) -> usize {
    match self {
      Self::ClearEdges => 0,
      Self::Balanced => 1,
      Self::SubtleEdges => 2,
    }
  }
}
