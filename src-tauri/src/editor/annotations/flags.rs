// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Flags carried by native annotation draw records, the twins of the bit
//! tests in the annotation shaders. The bits are ABI: never renumber one.

// Reserved so every kind reads the same bits: `FILL` for the rectangle and
// ellipse tools, unused until those tools land, and `MULTIPLY`, which no kind
// reads. `PIXELATE` marks a pixelated redaction, `MOSAIC` a classic
// pixelated one and `BLUR` a blurred one, or a spotlight that blurs what is
// outside it. `SURFACES` marks a redaction whose
// `p1` points at a surface timeline in the side buffer. `HAND_DRAWN` marks a
// highlight drawn as a marker stroke rather than a clean band, and
// `LAID_BY_HAND` one laid over a box, whose bands are strokes rather than
// lines of text. `SHADOW` marks a magnifier whose loupe casts a shadow, and a
// sticker that casts one. `ONCE` marks a sticker whose animation plays
// through once rather than looping, which only the sticker atlas reads.
#![allow(dead_code)]

pub(crate) const FILL: u32 = 1 << 0;
pub(crate) const MULTIPLY: u32 = 1 << 1;
pub(crate) const PIXELATE: u32 = 1 << 2;
pub(crate) const BLUR: u32 = 1 << 3;
pub(crate) const MOSAIC: u32 = 1 << 4;
pub(crate) const SURFACES: u32 = 1 << 5;
pub(crate) const HAND_DRAWN: u32 = 1 << 6;
pub(crate) const SHADOW: u32 = 1 << 7;
pub(crate) const LAID_BY_HAND: u32 = 1 << 8;
pub(crate) const ONCE: u32 = 1 << 9;
