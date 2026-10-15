// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral state for on-screen controls (OSC), and the GPU renderer
//! that draws them. Rendering backends can consume this model without
//! duplicating geometry or pointer semantics. The app hosts them: binding
//! windows, routing native input to its features, and owning the cursor.
pub mod controller;
pub mod controls;
pub mod cursor;
pub mod desktop;
pub mod geometry;
pub mod gesture;
pub mod gpu;
pub mod protocol;
mod resize;
pub mod scene;
pub mod semantic;
pub mod session;
pub mod style;
pub mod turn;

pub mod shortcut;
