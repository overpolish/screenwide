// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral state for on-screen controls (OSC), and the GPU renderer
//! that draws them. Rendering backends can consume this model without
//! duplicating geometry or pointer semantics. The app hosts them: binding
//! windows, routing native input to its features, and owning the cursor.
pub mod controller;
pub mod cursor;
pub mod desktop;
pub mod geometry;
pub mod gesture;
pub mod gpu;
pub mod protocol;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod precompiled_shaders {
  include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));
}
mod resize;
pub mod scene;
pub mod semantic;
pub mod session;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod shader_layouts;
pub mod style;
pub mod turn;

pub mod shortcut;
