// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <AppKit/AppKit.h>
#include <stdint.h>

#import "gpu_compositor_macos_annotation_text.h"

/// A text box's type: its face, how a block of it is measured, and how it is
/// rasterised into the annotation atlas.
///
/// The block is laid out one line per line break at a fixed line height, so
/// its height is exact, the Rust side can size the box from the widths alone,
/// and the editor's text view lays out the same lines at the same places.

/// A line's height, in ems. The twin of `LINE_HEIGHT` in
/// `annotations/text/metrics.rs`.
static const CGFloat SCREENWIDE_TEXT_BOX_LINE_HEIGHT = 1.25;

/// Inter SemiBold at `size`, the face counters' numbers and text boxes share;
/// `tabular` sets its figures to one width.
NSFont *screenwide_annotation_font(CGFloat size, BOOL tabular);

/// The paragraph a text box's lines are set in: `alignment` is the box's own
/// (0 left, 1 centre, 2 right) and every line is exactly the fixed line
/// height at `size`.
NSParagraphStyle *screenwide_text_box_paragraph(CGFloat size, uint32_t alignment);

/// One line's typographic width at `font`, for the Rust side to size a box
/// from. Called from any thread.
double screenwide_text_box_line_width(const uint8_t *text, uint32_t length, double font);

/// The atlas cell a text block needs at `size` drawn pixels, margin included,
/// in atlas pixels. Zero when there is nothing to draw.
uint32_t screenwide_text_box_cell(const char *text, uint32_t length, float size,
                                  uint32_t *width, uint32_t *height);

/// Rasterises a text block into its cell of the atlas, lines lined up by
/// `alignment` across the block's own width.
void screenwide_text_box_draw(uint8_t *pixels, uint32_t atlas_width,
                              ScreenwideAnnotationTextRect rect, const char *text,
                              uint32_t length, float size, uint32_t alignment);
