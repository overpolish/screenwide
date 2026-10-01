// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotation documents and editing, shared by still and timed workspaces.
//!
//! A tool's shape lives in one module under this one ([`arrow`],
//! [`counter`], [`text`], [`redact`], [`highlight`], [`outline`] for the
//! shape tool, [`spotlight`], [`freehand`] for the draw tool, [`magnify`])
//! and nothing else branches on which kind an annotation is.
//! Adding a tool is therefore:
//!
//! - one module here, with the `model`, `gesture`, `handles`, `native`,
//!   `snap`, `reveal` and `geometry` parts the kind needs;
//! - one arm in each `match` in [`shape`], which is the exhaustive list of
//!   what a kind has to answer;
//! - one `EDITOR_TOOLS` row and one `ANNOTATION_KINDS` row in TypeScript,
//!   plus its icon;
//! - the per-platform draw code that cannot be shared: a `geometry.h` twin, a
//!   Metal layer function and a WGSL layer function.

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
/// What moving a box's grips does to it, shared by the redaction, the shape
/// and the spotlight.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod box_gesture;
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
/// The draw tool's own half of the model: a line drawn freehand. Not called
/// `draw`, which is what every kind's native record is built by.
pub(crate) mod freehand;
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
/// Several annotations chosen together: the box round them and the move
/// that carries them all.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod group;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// The highlight tool's own half of the model.
pub(crate) mod highlight;
/// The magnifier tool's own half of the model: a loupe showing a zoom area
/// enlarged.
pub(crate) mod magnify;
/// What a marquee band chooses.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod marquee;
/// The shape tool's own half of the model: an outline round a box. Not
/// called `shape`, which is the enum every kind is a variant of.
pub(crate) mod outline;
/// How long an annotation's path takes to draw in, from its length: the
/// editor's pace, for the clips made natively.
pub(crate) mod pace;
/// Annotations on a recording following the content they were placed on.
pub(crate) mod pin;
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
/// The spotlight tool's own half of the model: a box left bright while the
/// picture around it dims.
pub(crate) mod spotlight;
/// The text tool's own half of the model.
pub(crate) mod text;

pub(crate) mod flags;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;

pub(crate) mod reveal;

pub(crate) mod timing;
