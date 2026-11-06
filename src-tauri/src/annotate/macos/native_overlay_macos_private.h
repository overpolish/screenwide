// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

#pragma once

#import <AppKit/AppKit.h>

/// What the overlay's two halves share: the surfaces, which the input half
/// asks about, and nothing else.

/// Whether any display has a surface.
BOOL screenwide_annotate_has_surfaces(void);

/// Whether `window` is one of the host windows the overlay draws on.
BOOL screenwide_annotate_owns_window(NSWindow *window);
