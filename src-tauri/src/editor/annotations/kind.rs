// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which shape an annotation is, as one value every layer of the stack shares.

use serde::{Deserialize, Serialize};

/// The kinds the compositor draws. The numbers are ABI: they are the C
/// `ScreenwideAnnotationKind`, the `SCREENWIDE_ANNOTATION_*` the Metal
/// compositor prepares against, and the `kind == 1u` tests in the Metal and
/// HLSL annotation shaders. They travel inside the native records as they are,
/// so a value may never be renumbered. The serde names are the `kind` tag of
/// `AnnotationShape` and what the live overlay's settings store, so a kind
/// is one word in a document, in settings, and over the native boundary.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationKind {
  #[default]
  Arrow = 0,
  Counter = 1,
  Text = 2,
  Redact = 3,
}

impl AnnotationKind {
  /// The kind a native record carries, or `None` for a number no kind owns.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn from_raw(kind: u32) -> Option<Self> {
    match kind {
      0 => Some(Self::Arrow),
      1 => Some(Self::Counter),
      2 => Some(Self::Text),
      3 => Some(Self::Redact),
      _ => None,
    }
  }

  /// The number the native records and the shaders read.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn raw(self) -> u32 {
    self as u32
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn kinds_read_back_from_their_native_numbers() {
    assert_eq!(AnnotationKind::from_raw(0), Some(AnnotationKind::Arrow));
    assert_eq!(AnnotationKind::from_raw(1), Some(AnnotationKind::Counter));
    assert_eq!(AnnotationKind::from_raw(2), Some(AnnotationKind::Text));
    assert_eq!(AnnotationKind::Arrow.raw(), 0);
    assert_eq!(AnnotationKind::Counter.raw(), 1);
    assert_eq!(AnnotationKind::Text.raw(), 2);
    assert_eq!(AnnotationKind::from_raw(3), Some(AnnotationKind::Redact));
    assert_eq!(AnnotationKind::Redact.raw(), 3);
    assert_eq!(AnnotationKind::from_raw(4), None);
  }
}
