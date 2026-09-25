// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotation documents and editing, shared by still and timed workspaces.
//!
//! A tool's shape lives in one module under this one ([`arrow`],
//! [`counter`], [`text`], [`redact`]) and nothing else branches on which
//! kind an annotation is.
//! Adding a tool is therefore:
//!
//! - one module here, with the `model`, `gesture`, `handles`, `native`,
//!   `snap`, `reveal` and `geometry` parts the kind needs;
//! - one arm in each `match` in [`shape`], which is the exhaustive list of
//!   what a kind has to answer;
//! - one `EDITOR_TOOLS` row and one `ANNOTATION_KINDS` row in TypeScript,
//!   plus its icon;
//! - the per-platform draw code that cannot be shared: a `geometry.h` twin, a
//!   Metal layer function and an HLSL layer function.

pub(crate) mod kind;
pub use kind::AnnotationKind;

pub(crate) mod model;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use model::annotation_colour;
pub(crate) use model::{
  Annotation, AnnotationAlign, AnnotationHead, AnnotationPoint, AnnotationRedaction,
  AnnotationStyle,
};

/// What an annotation is, and every per-kind branch there is.
pub(crate) mod shape;
pub(crate) use shape::AnnotationShape;

/// The arrow tool's own half of the model.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod arrow;
/// The counter tool's own half of the model.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod counter;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod edit;
#[cfg(test)]
mod edit_tests;
/// How far an annotation's picture moves while the shutter is open, which
/// both backends spread their exposure samples along.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod exposure;
/// The C entry points the Metal compositor and the macOS chrome reach a
/// kind's prepared geometry through.
#[cfg(target_os = "macos")]
pub(crate) mod ffi;
/// The draw record every kind fills, and the arithmetic it is built with.
/// Both backends prepare from it: the D3D11 one calls each kind's `geometry`
/// module, and the Metal compositor and the macOS chrome reach the same
/// functions through `ffi`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
#[cfg(test)]
mod gesture_tests;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// The redaction tool's own half of the model.
pub(crate) mod redact;
/// Where a gesture's positions land while the positional modifier is held.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod snap;
#[cfg(test)]
mod snap_edit_tests;
#[cfg(test)]
mod snap_gap_tests;
#[cfg(test)]
mod snap_tail_tests;
/// The text tool's own half of the model.
pub(crate) mod text;

pub(crate) mod flags;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;

pub(crate) mod reveal;

pub(crate) mod timing;
