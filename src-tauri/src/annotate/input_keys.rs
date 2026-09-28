// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keys the overlay answers to, by the codes each platform reports.

use crate::editor::annotations::AnnotationKind;

/// Virtual key codes, which AppKit reports by physical position.
#[cfg(target_os = "macos")]
mod key_codes {
  pub(in super::super) const KEY_A: u16 = 0;
  pub(in super::super) const KEY_H: u16 = 4;
  pub(in super::super) const KEY_Z: u16 = 6;
  pub(in super::super) const KEY_N: u16 = 45;
  pub(in super::super) const KEY_BACKSPACE: u16 = 51;
  pub(in super::super) const KEY_FORWARD_DELETE: u16 = 117;
}

/// Windows virtual key codes, which name the character rather than the
/// position it sits at.
#[cfg(target_os = "windows")]
mod key_codes {
  pub(in super::super) const KEY_A: u16 = 0x41;
  pub(in super::super) const KEY_H: u16 = 0x48;
  pub(in super::super) const KEY_Z: u16 = 0x5A;
  pub(in super::super) const KEY_N: u16 = 0x4E;
  pub(in super::super) const KEY_BACKSPACE: u16 = 0x08;
  pub(in super::super) const KEY_FORWARD_DELETE: u16 = 0x2E;
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
use key_codes::{KEY_A, KEY_H, KEY_N};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) use key_codes::{KEY_BACKSPACE, KEY_FORWARD_DELETE, KEY_Z};

/// The tool each unmodified letter picks up, the twin of the toolbar's own
/// hints. A shape added to the overlay takes its letter here.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) const TOOL_KEYS: &[(u16, AnnotationKind)] = &[
  (KEY_A, AnnotationKind::Arrow),
  (KEY_N, AnnotationKind::Counter),
  (KEY_H, AnnotationKind::Highlight),
];
