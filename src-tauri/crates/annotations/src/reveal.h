// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

/// The reveal a timed annotation draws itself in and out through. The maths is
/// in Rust - `src-tauri/src/editor/annotations/reveal.rs` - so the native
/// preview, the still export and the video export all animate through one
/// implementation; this is its twin.

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
