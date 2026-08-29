// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral state for on-screen controls (OSC). Rendering backends can
//! consume this model without duplicating geometry or pointer semantics.
pub mod controller;
pub mod desktop;
pub mod geometry;
pub mod gesture;
mod resize;
pub mod style;
