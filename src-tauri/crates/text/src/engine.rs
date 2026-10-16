// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How type is set on each platform: the faces it is set in, how much room a
//! block of it takes, and typing into it where the platform has no text view.
//! Core Text sets it on macOS, from `engine/macos`, and DirectWrite on
//! Windows.

/// The families the templates bring, read by the platform text engines.
pub mod app_fonts;
/// Which face type is set in: Inter for annotations, any family for styled
/// text.
pub mod face;
/// The font families installed on the system.
pub mod installed;
/// How much room a block of text takes, asked of the platform's text engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod metrics;
/// Inter set by DirectWrite on Windows: the face annotation type is measured
/// and rasterised with there. macOS sets it with Core Text, in `macos/`.
#[cfg(target_os = "windows")]
pub mod type_device;
/// Typing where the platform has no text view to lay over the text: the
/// text, the selection and every edit a key makes. macOS types into a native
/// text view and reads only the marks the shader draws.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[cfg_attr(target_os = "macos", allow(dead_code, unused_imports))]
pub mod typing;
