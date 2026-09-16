// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotation documents and editing, shared by still and timed workspaces.

pub(crate) mod model;
#[cfg(target_os = "macos")]
pub(crate) use model::annotation_colour;
pub(crate) use model::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};
#[cfg(any(target_os = "macos", test))]
pub(crate) mod bend;
#[cfg(any(target_os = "macos", test))]
pub(crate) mod edit;
#[cfg(any(target_os = "macos", test))]
pub(crate) mod gesture;
#[cfg(test)]
mod gesture_tests;
#[cfg(target_os = "macos")]
pub(crate) mod handles;

/// The native retained scene, export and the live overlay accept the same
/// number of marks.
pub(crate) const MAX_ANNOTATIONS: usize = 32;

#[cfg(target_os = "macos")]
pub(crate) mod native;

pub(crate) mod reveal;

pub(crate) mod timing;
