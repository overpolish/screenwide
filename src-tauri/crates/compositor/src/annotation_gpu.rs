// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation shader draws, prepared on the CPU: the storage-buffer
//! elements, a layer's annotations placed into them, and the atlases their
//! type and their images' pictures are drawn in. The editor's compositor
//! and the live annotate overlay both draw from these, each with buffers and
//! a type atlas of its own; only the editor draws images.

mod arrows;
/// Where the caption showing is drawn, for hit-testing.
mod caption_bounds;
/// Caption pieces as text boxes, placed as the arrows they already are.
mod caption_boxes;
/// A caption piece blurred over the frame's exposure.
mod caption_exposure;
/// The shadow a caption's word casts, handed to the shader.
mod caption_shadow;
/// A caption piece turned flat or in depth, handed to the shader.
mod caption_turn;
/// The captions showing in a frame, drawn as text boxes.
mod captions;
mod counter_artwork;
/// A focus box, held beside the arrows for the blur.
mod focus_region;
/// An atlas's texture.
mod gpu_atlas;
mod image_artwork;
mod prepare;
/// The caret and selection of text typed into word by word.
mod typing_bars;

pub use arrows::{
  GpuBuffer, PreparedArrows, PreparedImage, PreparedType, PreviewArrow, PreviewSample,
  MAX_EXPOSURE_SAMPLES,
};
pub use caption_bounds::{caption_editing_block, caption_visible_bounds};
pub use captions::{
  caption_arrows, caption_face, caption_measure, caption_overlay, caption_overlay_typed,
  CaptionFaces,
};
/// A counter's number and a text box's lines are type, so they are
/// rasterised rather than drawn by the shader.
pub use counter_artwork::{numbered_arrows, CounterAtlas};
pub use focus_region::{FocusRegion, GpuFocusRegion, FOCUS_REACH_SHARE, FOCUS_WIDEST_SHARE};
/// An image's picture is drawn into an atlas of its own, in colour.
pub(crate) use image_artwork::{place_images, ImageAtlas};
pub use prepare::SourceAnnotations;
pub use prepare::{placed_arrows, prepared_arrows};
