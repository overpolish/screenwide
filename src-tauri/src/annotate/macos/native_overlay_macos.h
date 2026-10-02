// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

/// Modifier bits carried with a key press. Named here rather than passing
/// AppKit's flags through, so the Rust side never depends on AppKit.
#define SCREENWIDE_ANNOTATE_MODIFIER_COMMAND 1u
#define SCREENWIDE_ANNOTATE_MODIFIER_SHIFT 2u
#define SCREENWIDE_ANNOTATE_MODIFIER_OPTION 4u
#define SCREENWIDE_ANNOTATE_MODIFIER_CONTROL 8u

/// A pointer step: 0 down, 1 drag, 2 up. The point is in global desktop
/// points with y running down, the space the cursor sidecar reports in.
typedef void (*ScreenwideAnnotatePointer)(uint32_t phase, double x, double y);

/// A key press. Returns non-zero when the overlay acted on it.
typedef uint32_t (*ScreenwideAnnotateKey)(uint16_t key_code,
                                          uint32_t modifiers);

/// Gives one host window's view a `CAMetalLayer` for Rust to draw into, and
/// returns it; NULL when the view cannot take one. Main thread only.
void *screenwide_annotate_attach(void *view_ptr);

/// Drops the layer and forgets the surface. Main thread only.
void screenwide_annotate_detach(void *view_ptr);

/// Redraws every attached display, in Rust. Main thread only.
void screenwide_annotate_redraw(void);

/// Starts swallowing pointer and key events and routing them to Rust. The
/// overlay has no pass-through mode: while input is installed, nothing under
/// it sees any. Main thread only.
void screenwide_annotate_install_input(ScreenwideAnnotatePointer pointer,
                                       ScreenwideAnnotateKey key);

/// Stops swallowing input. Main thread only.
void screenwide_annotate_teardown_input(void);

/// Whether the pointer over the canvas is the I-beam, for the tool that selects
/// text, rather than the crosshair. Main thread only.
void screenwide_annotate_set_text_cursor(uint32_t text);
