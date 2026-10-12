// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The app's hosts for on-screen controls: the runtime that binds a window to
//! a controller and routes its input to the Region, Text Recognition and
//! Ruler features, the Windows keyboard hook that feeds them, and the macOS
//! cursor they share. The controls themselves are `screenwide_osc`.

#[cfg(target_os = "macos")]
pub(crate) mod cursor_macos;
#[cfg(target_os = "windows")]
pub(crate) mod keyboard_windows;
pub mod runtime;
