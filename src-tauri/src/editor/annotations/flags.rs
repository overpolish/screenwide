// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Flags carried by native annotation draw records.

#![allow(dead_code)]
// Reserved for the compositing tools that consume the retained side buffers.
pub(crate) const FILL: u32 = 1 << 0;
pub(crate) const MULTIPLY: u32 = 1 << 1;
pub(crate) const PIXELATE: u32 = 1 << 2;
