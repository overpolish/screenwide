// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral state for on-screen controls (OSC). Rendering backends can
//! consume this model without duplicating geometry or pointer semantics.
pub mod controller;
pub mod controls;
pub mod desktop;
pub mod geometry;
pub mod gesture;
pub mod protocol;
mod resize;
pub mod runtime;
pub mod scene;
pub mod semantic;
pub mod session;
pub mod style;
