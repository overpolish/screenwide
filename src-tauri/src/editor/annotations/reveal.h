// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

/// The reveal a timed annotation draws itself in and out through. The maths is
/// in Rust - `src-tauri/src/editor/annotations/reveal.rs` - so the native
/// preview, the still export and the video export all animate through one
/// implementation; these are its twins and its entry points.

/// Arc-length window, size and opacity, plus the shutter-start state.
typedef struct {
  float low, high, scale, opacity;
  float previous[4];
} AnnotationReveal;

/// The whole path at full size, standing still: a screenshot, an annotation
/// that does not animate, and the hold between a clip's two phases.
static inline AnnotationReveal annotation_reveal_whole(void) {
  return (AnnotationReveal){0, 1, 1, 1, {0, 1, 1, 1}};
}

/// One prepared exposure sample in curve parameters and canvas pixels.
typedef struct {
  float low, high, start_tip, end_tip, scale;
} AnnotationRevealGeometry;

/// The window a clip is at, `elapsed_ms` into a clip lasting `duration_ms`.
/// `frame_ms` is the exposure interval in source time; a still passes zero.
/// `kind` is the annotation's own, because a counter arrives on its own timing.
void screenwide_annotation_reveal_window(float elapsed_ms, float duration_ms,
                                         float frame_ms, uint32_t animated,
                                         uint32_t kind, AnnotationReveal *out);

/// The prepared reveal for one annotation, in the space its points were given
/// in. `stroke` is the full stroke width and `heads` the number of arrowheads.
void screenwide_annotation_reveal_geometry(float ax, float ay, float bx,
                                           float by, float cx, float cy,
                                           float stroke, float heads,
                                           AnnotationReveal window,
                                           AnnotationRevealGeometry *out);
