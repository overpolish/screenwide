// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <AppKit/AppKit.h>
#include <stdint.h>

/// A text box's type: its face, and how a line of it is measured.
///
/// The block is laid out one line per line break at a fixed line height, so
/// its height is exact, the Rust side can size the box from the widths alone,
/// and the editor's text view lays out the same lines at the same places.

/// A line's height, in ems. The twin of `LINE_HEIGHT` in the annotations
/// crate's `text/metrics.rs`.
static const CGFloat SCREENWIDE_TEXT_BOX_LINE_HEIGHT = 1.25;

/// Inter SemiBold at `size`, the face counters' numbers and text boxes share;
/// `tabular` sets its figures to one width.
NSFont *screenwide_annotation_font(CGFloat size, BOOL tabular);

/// `family` at `weight` on the CSS scale, at `size`: a family installed on
/// the system, or the app's Inter for none or for one no longer installed.
NSFont *screenwide_face_font(NSString *family, uint32_t weight, CGFloat size);

/// [`screenwide_face_font`] with the family as UTF-8 bytes, none for empty.
NSFont *screenwide_face_font_bytes(const uint8_t *family, uint32_t length, uint32_t weight,
                                   CGFloat size);

/// The paragraph a text box's lines are set in: `alignment` is the box's own
/// (0 left, 1 centre, 2 right) and every line is exactly the fixed line
/// height at `size`.
NSParagraphStyle *screenwide_text_box_paragraph(CGFloat size, uint32_t alignment);

/// One line's typographic width in a face at `font`, for the Rust side to
/// size a box from. Called from any thread.
double screenwide_face_line_width(const uint8_t *text, uint32_t length, double font,
                                  const uint8_t *family, uint32_t family_length, uint32_t weight);
