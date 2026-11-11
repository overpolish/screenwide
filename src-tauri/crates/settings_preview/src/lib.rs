// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Live pictures inside a window, for a page that shows what a setting does:
//! a GPU surface placed over a box in the page, clipped, rounded and faded
//! with it, drawn by the app's own renderers. It shows either a tool at work
//! on a picture the page paints, driven by the page's pointer, which the app
//! crate draws with the tool's overlay renderer, or a recording of the
//! page's pictures with a scripted pointer, composed here by the editor's
//! compositor exactly as an export composes a frame: its cursor, its auto
//! zooms, its frame rate.

pub mod placement;
pub mod scene;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod host;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod preview;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod precompiled_shaders {
  include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));
}
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod shader_layouts;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod worker;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use host::{HostWindow, Stacking};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use preview::Preview;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use worker::{Message, Renderer};
