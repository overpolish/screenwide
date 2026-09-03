// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared GPU OSC rendering primitives.
//!
//! Tool surfaces supply semantic scene geometry; Region, Export and future
//! overlays consume the same platform renderer from here.

#[cfg(target_os = "windows")]
pub(crate) mod windows;
