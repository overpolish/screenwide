// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What each key cap shows: the key's name as the platform's keyboards print
//! it, or for the modifiers the React keycap draws as Lucide icons, that
//! icon.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod windows;

#[cfg(target_os = "macos")]
use macos::key_label;
#[cfg(not(target_os = "macos"))]
use windows::key_label;

/// Command and Control are icons only on the Mac.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Icon {
  /// Lucide Command.
  Command,
  /// Lucide ArrowBigUp.
  Shift,
  /// Lucide ChevronUp.
  Control,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Cap {
  Text(String),
  Icon(Icon),
}

pub(super) fn cap(code: u16) -> Cap {
  icon(code).map_or_else(|| Cap::Text(key_label(code)), Cap::Icon)
}

/// The Mac's modifier symbols are icons; Windows names Ctrl, Alt and Win,
/// and only Shift keeps its arrow.
#[cfg(target_os = "macos")]
fn icon(code: u16) -> Option<Icon> {
  match code {
    54 | 55 => Some(Icon::Command),
    56 | 60 => Some(Icon::Shift),
    59 | 62 => Some(Icon::Control),
    _ => None,
  }
}

#[cfg(not(target_os = "macos"))]
fn icon(code: u16) -> Option<Icon> {
  matches!(code, 56 | 60).then_some(Icon::Shift)
}
