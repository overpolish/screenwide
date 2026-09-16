// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

#import "../editor/cursor_export/gpu_compositor_macos_annotation_types.h"

/// Modifier bits carried with a key press. Named here rather than passing
/// AppKit's flags through, so the Rust side never depends on AppKit.
#define SCREENWIDE_ANNOTATE_MODIFIER_COMMAND 1u
#define SCREENWIDE_ANNOTATE_MODIFIER_SHIFT 2u
#define SCREENWIDE_ANNOTATE_MODIFIER_OPTION 4u
#define SCREENWIDE_ANNOTATE_MODIFIER_CONTROL 8u

/// What one display's overlay draws: the annotations on screen plus the stroke in
/// hand, already in that display's layer pixels.
typedef void (*ScreenwideAnnotateScene)(uint32_t display,
                                        ScreenwideAnnotations *out);

/// A pointer step: 0 down, 1 drag, 2 up. The point is in global desktop
/// points with y running down, the space the cursor sidecar reports in.
typedef void (*ScreenwideAnnotatePointer)(uint32_t phase, double x, double y);

/// A key press. Returns non-zero when the overlay acted on it.
typedef uint32_t (*ScreenwideAnnotateKey)(uint16_t key_code,
                                          uint32_t modifiers);

/// Gives one host window's view a Metal layer that draws `display`'s annotations.
/// Returns non-zero on success. Main thread only.
uint32_t screenwide_annotate_attach(void *view_ptr, uint32_t display);

/// Drops the layer and forgets the surface. Main thread only.
void screenwide_annotate_detach(void *view_ptr);

/// Compiles the overlay's Metal library and reports the outcome: 1 built, 2
/// no Metal device on this machine, 0 failed with the reason in `message`.
/// The kernel is compiled from source at runtime, so nothing else would catch
/// a shader that no longer builds until an overlay opened blank.
uint32_t screenwide_annotate_shader_check(char *message, uint32_t capacity);

/// Redraws every attached display. Main thread only.
void screenwide_annotate_redraw(void);

/// Names the source of the annotations every display draws. Outlives input: annotations
/// kept after exiting are still drawn by hosts that take no input at all.
/// Main thread only.
void screenwide_annotate_install_scene(ScreenwideAnnotateScene scene);

/// Starts swallowing pointer and key events and routing them to Rust. The
/// overlay has no pass-through mode: while input is installed, nothing under
/// it sees any. Main thread only.
void screenwide_annotate_install_input(ScreenwideAnnotatePointer pointer,
                                       ScreenwideAnnotateKey key);

/// Stops swallowing input. Main thread only.
void screenwide_annotate_teardown_input(void);
