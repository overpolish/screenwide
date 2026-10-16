// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which face type is set in. Annotations always use the app's own Inter
//! SemiBold; captions may ask for any family installed on the system or
//! carried by the app, at any weight, upright or italic, and fall back to
//! Inter where the family is missing.

/// A family, a weight on the CSS scale of 100 to 900, and whether italic.
/// No family is the app's own Inter.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeFace {
  pub family: Option<String>,
  pub weight: u16,
  /// Set in the family's italic, where it has one; upright where not.
  pub italic: bool,
}

impl Default for TypeFace {
  fn default() -> Self {
    Self::ANNOTATION
  }
}
impl TypeFace {
  /// The face annotation type is set in.
  pub const ANNOTATION: Self = Self {
    family: None,
    weight: 600,
    italic: false,
  };

  /// The face, with no family where it names none.
  pub fn new(family: Option<&str>, weight: u16, italic: bool) -> Self {
    Self {
      family: family
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(str::to_owned),
      weight: weight.clamp(100, 900),
      italic,
    }
  }

  /// The family as UTF-8 bytes for a native call, empty for Inter.
  pub fn family_bytes(&self) -> &[u8] {
    self.family.as_deref().unwrap_or_default().as_bytes()
  }
}
