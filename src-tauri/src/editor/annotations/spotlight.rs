// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spotlight: a box left bright while everything around it dims.
//!
//! A spotlight is not a mark on the picture but a hole in a shade over it.
//! Every spotlight showing at once cuts its hole in one shared shade, so two
//! of them light two places rather than darkening each other, and the shade
//! is as dark as the darkest of them. The shade goes over the picture and
//! darkens the cursor, which lies on it, and goes under every mark, so an
//! arrow pointing into the light is not dimmed on its way. How dark the shade
//! is is not a choice: [`model::SPOTLIGHT_DIM`].
//!
//! Blur is the one option beyond the box: what lies outside the light is
//! also softened. It is applied to the source the way a blurred redaction
//! is, as one extra pass over the whole picture with every hole cut out of
//! it, so the crop and the magnifier see it too, and to the cursor by the
//! same deviation.
//!
//! The box is held and edited exactly as a redaction's or a shape's is,
//! through [`super::box_gesture`].

/// Draw-ready geometry and how much light reaches a point, prepared for both
/// backends.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// One spotlight handing its light to the next butted up against it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod handoff;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record, and the blur pass it asks the source for.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod native;
/// How a spotlight fades in and out over its clip.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;
/// A picture softened as the blur softens one, for the live overlay.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod soften;

#[cfg(test)]
mod tests;
