// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation shader draws, prepared on the CPU: the storage-buffer
//! elements, a layer's annotations placed into them, and the atlas their
//! type is set in. The editor's compositor and the live annotate overlay
//! both draw from these, each with buffers and an atlas of its own.

mod arrows;
mod counter_artwork;
mod prepare;

pub(crate) use arrows::{
  GpuBuffer, PreparedArrows, PreparedType, PreviewArrow, PreviewSample, MAX_EXPOSURE_SAMPLES,
};
/// A counter's number and a text box's lines are type, so they are
/// rasterised rather than drawn by the shader.
pub(crate) use counter_artwork::{numbered_arrows, CounterAtlas};
#[cfg(target_os = "macos")]
pub(crate) use prepare::SourceAnnotations;
pub(crate) use prepare::{placed_arrows, prepared_arrows};
