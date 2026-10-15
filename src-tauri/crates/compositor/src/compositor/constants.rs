// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the canvas pass is told for one draw, laid out as `preview.wgsl`
//! reads it.

/// The twin of `Canvas` in `preview.wgsl`; every member is a 16-byte row.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Constants {
  pub(super) output_source: [f32; 4],
  pub(super) image_rect: [f32; 4],
  pub(super) crop_rect: [f32; 4],
  pub(super) source_crop_rect: [f32; 4],
  /// The crop tool's result layer: its output-pixel rectangle, then its
  /// radius, an enabled flag, the shadow sigma and one spare word.
  pub(super) crop_preview_rect: [f32; 4],
  pub(super) crop_preview_effects: [f32; 4],
  pub(super) solid_color: [f32; 4],
  pub(super) base_color: [f32; 4],
  pub(super) recenter_inset_color: [f32; 4],
  pub(super) mesh_points: [[f32; 4]; 8],
  pub(super) mesh_colors: [[f32; 4]; 4],
  pub(super) effects: [f32; 4],
  /// Timeline seconds, the generator speed the ported generators scale them
  /// by, how many canvas pixels one drawn pixel covers, and how many atlas
  /// pixels the annotations' type holds per canvas pixel.
  pub(super) motion: [f32; 4],
  pub(super) cursor_geometry: [f32; 4],
  pub(super) cursor_effects: [f32; 4],
  pub(super) cursor_blur: [f32; 4],
  pub(super) camera_frame: [f32; 4],
  pub(super) camera_crop: [f32; 4],
  pub(super) camera_effects: [f32; 4],
  pub(super) magnifier: [f32; 4],
  pub(super) magnifier_options: [f32; 4],
  pub(super) magnifier_bounds: [f32; 4],
  pub(super) native_cursor_hotspots: [[f32; 4]; 8],
  pub(super) options: [u32; 4],
  pub(super) cursor_options: [u32; 4],
  pub(super) background_options: [u32; 4],
  /// Annotations below the camera are sorted ahead of those above it, so the
  /// first word is both the below-camera count and where the above-camera run
  /// starts; the second is the total. The rest are spare.
  pub(super) annotation_options: [u32; 4],
  /// The handed artwork's bitmap and design size, the hotspot in the
  /// recorded cursor box and the design's origin, and the artwork flags.
  pub(super) cursor_artwork: [f32; 4],
  pub(super) cursor_frame: [f32; 4],
  pub(super) cursor_model: [u32; 4],
  /// The annotation tiles: side, grid width and height, and words a tile's
  /// set takes.
  pub(super) annotation_tiles: [u32; 4],
  /// Where the canvas is drawn in its target: the corner, and the canvas
  /// pixels one target pixel covers. A canvas drawn at its own size from the
  /// target's corner carries the identity.
  pub(super) placement: [f32; 4],
  /// The spotlights' blur of the annotations under their shade: the layer's
  /// texels per canvas pixel, whether this pass draws that layer (1) or reads
  /// it (2), and the index of the spotlight whose shade it lies under.
  pub(super) annotation_blur: [f32; 4],
  /// How a scene moved the screen's box while the shutter was open: the scale
  /// and the shift, in canvas pixels, that carry the box as drawn onto where it
  /// was, then how many steps the frame averages. One step draws it sharp.
  pub(super) scene_motion: [f32; 4],
  /// The same scale and shift for the screen's image, which a zoom carries
  /// further than its box.
  pub(super) scene_image_motion: [f32; 4],
  /// The camera's frame when the shutter opened, in canvas pixels, and the
  /// part of its picture that frame showed, in shares of the picture.
  pub(super) camera_motion_frame: [f32; 4],
  pub(super) camera_motion_crop: [f32; 4],
  /// How opaque the screen and the camera are drawn, which a scene fades as
  /// it hides or shows them; the other two are unused.
  pub(super) scene_opacity: [f32; 4],
  /// What the pictures the screen and the camera are drawn from say of
  /// themselves: whether each is enlarged along its edges (FSR 1's EASU)
  /// where drawn larger than it is, then whether each is a person cut out,
  /// whose shadow follows their outline.
  pub(super) source_flags: [f32; 4],
  /// How a scene turns the screen in depth: the rows of the inverse of the
  /// turn, which carries a canvas pixel back to the point of the upright
  /// screen it shows, then the rows of the turn, then the inverse turn the
  /// shutter opened on, which a moving scene's steps blend toward. See
  /// `scene_turn.rs`.
  pub(super) scene_turn: [[f32; 4]; super::scene_turn::SCENE_TURN_ROWS],
  /// Which part of a focused frame this pass draws: the whole frame (0),
  /// only what lies under the topmost focus (1), or only what lies over it,
  /// on nothing (2); then where in the annotations that focus sits, and
  /// whether it lies in the run above the camera. See `focus.rs`.
  pub(super) focus: [f32; 4],
}
