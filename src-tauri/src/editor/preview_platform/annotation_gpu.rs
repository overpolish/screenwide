// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation shader draws, prepared on the CPU: the storage-buffer
//! elements, a layer's annotations placed into them, and the atlases their
//! type and their stickers' pictures are drawn in. The editor's compositor
//! and the live annotate overlay both draw from these, each with buffers and
//! a type atlas of its own; only the editor draws stickers.

mod arrows;
mod counter_artwork;
/// An atlas's texture.
mod gpu_atlas;
mod prepare;
mod sticker_artwork;

pub(crate) use arrows::{
  GpuBuffer, PreparedArrows, PreparedSticker, PreparedType, PreviewArrow, PreviewSample,
  MAX_EXPOSURE_SAMPLES,
};
/// A counter's number and a text box's lines are type, so they are
/// rasterised rather than drawn by the shader.
pub(crate) use counter_artwork::{numbered_arrows, CounterAtlas};
pub(crate) use prepare::SourceAnnotations;
pub(crate) use prepare::{placed_arrows, prepared_arrows};
/// A sticker's picture is drawn into an atlas of its own, in colour.
pub(crate) use sticker_artwork::{place_stickers, StickerAtlas};
