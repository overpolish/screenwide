// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Flags carried by native annotation draw records, the twins of the
//! `SCREENWIDE_ANNOTATION_FLAG_*` defines in
//! `cursor_export/gpu_compositor_macos_annotation_types.h` and of the bit
//! tests in both annotation shaders. The bits are ABI: never renumber one.

// Reserved so every kind reads the same bits: `FILL` for the rectangle and
// ellipse tools and `MULTIPLY` for the highlight, unused until those tools
// land. `PIXELATE` marks a pixelated redaction, `MOSAIC` a classic
// pixelated one and `BLUR` a blurred one. `SURFACES` marks a redaction whose
// `p1` points at a surface timeline in the side buffer.
#![allow(dead_code)]

pub(crate) const FILL: u32 = 1 << 0;
pub(crate) const MULTIPLY: u32 = 1 << 1;
pub(crate) const PIXELATE: u32 = 1 << 2;
pub(crate) const BLUR: u32 = 1 << 3;
pub(crate) const MOSAIC: u32 = 1 << 4;
pub(crate) const SURFACES: u32 = 1 << 5;
