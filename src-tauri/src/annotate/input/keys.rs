// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The keys the overlay answers to, by the codes each platform reports.

use screenwide_annotations::AnnotationKind;

/// Virtual key codes, which AppKit reports by physical position.
#[cfg(target_os = "macos")]
mod key_codes {
  pub(in super::super) const KEY_A: u16 = 0;
  pub(in super::super) const KEY_C: u16 = 8;
  pub(in super::super) const KEY_D: u16 = 2;
  pub(in super::super) const KEY_H: u16 = 4;
  pub(in super::super) const KEY_Z: u16 = 6;
  pub(in super::super) const KEY_O: u16 = 31;
  pub(in super::super) const KEY_S: u16 = 1;
  pub(in super::super) const KEY_BACKSPACE: u16 = 51;
  pub(in super::super) const KEY_FORWARD_DELETE: u16 = 117;
}

/// Windows virtual key codes, which name the character rather than the
/// position it sits at.
#[cfg(target_os = "windows")]
mod key_codes {
  pub(in super::super) const KEY_A: u16 = 0x41;
  pub(in super::super) const KEY_C: u16 = 0x43;
  pub(in super::super) const KEY_D: u16 = 0x44;
  pub(in super::super) const KEY_H: u16 = 0x48;
  pub(in super::super) const KEY_Z: u16 = 0x5A;
  pub(in super::super) const KEY_O: u16 = 0x4F;
  pub(in super::super) const KEY_S: u16 = 0x53;
  pub(in super::super) const KEY_BACKSPACE: u16 = 0x08;
  pub(in super::super) const KEY_FORWARD_DELETE: u16 = 0x2E;
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
use key_codes::{KEY_A, KEY_C, KEY_D, KEY_H, KEY_O, KEY_S};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) use key_codes::{KEY_BACKSPACE, KEY_FORWARD_DELETE, KEY_Z};

/// The tool each unmodified letter picks up, the twin of the island's tool
/// buttons and the same letter the editor gives the tool (`EDITOR_TOOLS` in
/// `src/features/editor/tools/tool-registry.ts`). A shape added to the overlay
/// takes its letter here. O picks up the focus, which dims as a spotlight or
/// not by its own setting.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) const TOOL_KEYS: &[(u16, AnnotationKind)] = &[
  (KEY_A, AnnotationKind::Arrow),
  (KEY_C, AnnotationKind::Counter),
  (KEY_H, AnnotationKind::Highlight),
  (KEY_S, AnnotationKind::Shape),
  (KEY_O, AnnotationKind::Focus),
  (KEY_D, AnnotationKind::Draw),
];
