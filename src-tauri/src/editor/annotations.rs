// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotation documents and editing, shared by still and timed workspaces.

pub(crate) mod model;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use model::annotation_colour;
pub(crate) use model::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod bend;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod counter;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod edit;
/// Draw-ready arrow geometry. The Metal compositor prepares its arrows
/// through `geometry.h`; the D3D11 one has no C to call into, so it prepares
/// them here against the same arithmetic.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod geometry;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
#[cfg(test)]
mod gesture_tests;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;

/// The native retained scene, export and the live overlay accept the same
/// number of marks.
pub(crate) const MAX_ANNOTATIONS: usize = 32;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;

pub(crate) mod reveal;

pub(crate) mod timing;
