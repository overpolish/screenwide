// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared GPU OSC rendering primitives.
//!
//! Tool surfaces supply semantic scene geometry; Region, Export and future
//! overlays consume the same renderer from here.

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod magnifier;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod pipeline;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod vertices;
