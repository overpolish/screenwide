// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Flags carried by native annotation draw records, the twins of the
//! `SCREENWIDE_ANNOTATION_FLAG_*` defines in
//! `cursor_export/gpu_compositor_macos_annotation_types.h` and of the bit
//! tests in both annotation shaders. The bits are ABI: never renumber one.

// Reserved now so every kind reads the same bits: `FILL` for the rectangle
// and ellipse tools, `MULTIPLY` for the highlight, `PIXELATE` for obfuscate.
// Unused until those tools land.
#![allow(dead_code)]

pub(crate) const FILL: u32 = 1 << 0;
pub(crate) const MULTIPLY: u32 = 1 << 1;
pub(crate) const PIXELATE: u32 = 1 << 2;
