// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas shader built for the annotation kinds a draw shows.
//!
//! A shader holding every kind is costly to run even where most of it is
//! never taken: the GPU compiler sizes each pixel's registers for the deepest
//! path it keeps, and the full canvas shader outgrows them, so every pixel of
//! every annotated frame paid for the memory it spilled to. Measured on a
//! recording's images, the canvas pass took about 6 ms a frame with every
//! kind compiled in and under 2 ms with images alone. Each variant is the
//! one source with its kinds named in a constant, which the compiler folds,
//! leaving the other kinds' code out.

use crate::editor::annotations::AnnotationKind;
use crate::editor::preview_platform::annotation_gpu::PreviewArrow;

use super::SHADER;

/// A set of annotation kinds, a bit each by [`AnnotationKind::raw`], as the
/// `annotation_kinds_drawn` constant in `annotations.wgsl` reads it.
pub(crate) type KindMask = u32;

/// The line every variant replaces, and the line with no annotation at all,
/// which also drops the annotation passes themselves.
const ALL_KINDS: &str = "const annotation_kinds_drawn: u32 = 0xffffffffu;";
const DRAWN: &str = "const annotations_drawn: bool = true;";
const LEFT_OUT: &str = "const annotations_drawn: bool = false;";

/// `kind` alone.
pub(crate) fn kind_bit(kind: AnnotationKind) -> KindMask {
  1 << kind.raw()
}

/// The kinds `annotations` show.
pub(super) fn kinds_of(annotations: &[PreviewArrow]) -> KindMask {
  annotations
    .iter()
    .filter_map(|annotation| AnnotationKind::from_raw(annotation.kind))
    .fold(0, |kinds, kind| kinds | kind_bit(kind))
}

/// Every kind there is: the variant holding them all draws any frame.
pub(super) fn every_kind() -> KindMask {
  (0..KindMask::BITS)
    .filter_map(AnnotationKind::from_raw)
    .fold(0, |kinds, kind| kinds | kind_bit(kind))
}

/// The canvas shader's text drawing `kinds`; with none, it draws no
/// annotation at all. Every kind is the shader as written, whose constant
/// holds every bit: the build compiles that text and the one with none
/// (`build/precompiled_shaders.rs`), and these must stay the same text.
pub(super) fn source(kinds: KindMask) -> String {
  debug_assert!(
    SHADER.contains(ALL_KINDS) && SHADER.contains(DRAWN),
    "the canvas shader declares the kinds it draws"
  );
  if kinds == 0 {
    return SHADER.replacen(DRAWN, LEFT_OUT, 1);
  }
  if kinds == every_kind() {
    return SHADER.to_owned();
  }
  SHADER.replacen(
    ALL_KINDS,
    &format!("const annotation_kinds_drawn: u32 = {kinds}u;"),
    1,
  )
}

/// Whether `kinds` holds every kind in `needed`.
pub(super) fn covers(kinds: KindMask, needed: KindMask) -> bool {
  kinds & needed == needed
}
