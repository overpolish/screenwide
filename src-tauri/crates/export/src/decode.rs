// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading a recording's frames: whole frames for stills and the export,
//! brightness-only frames for pinned annotations' tracking, and on Windows
//! frames decoded straight into Direct3D 11 textures.

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub use macos::source_frame::{each_source_frame, source_frame_image};
#[cfg(target_os = "macos")]
pub use macos::video::LumaReader;
#[cfg(target_os = "windows")]
pub use windows::decoder::LumaReader;
#[cfg(target_os = "windows")]
pub use windows::source_frames::{each_source_frame, source_frame_image};
