// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the editor workspace's input speaks on every platform: the layer a
//! selection outlines and the gestures it reports. The input a platform
//! hands in is in `input`.

pub type SelectionCallback = Box<dyn FnMut(Option<u32>) + Send + 'static>;
pub type PointerPressCallback = Box<dyn FnMut(PointerPress) + Send + 'static>;

/// A press on the picture as the editor above hears of it, and the pointer
/// coming onto the picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPress {
  /// A button went down. `took` says whether the press took hold of
  /// something already there (a layer, an annotation, a grip or a ring) or
  /// began Select's band, rather than drawing or panning: either is Select's
  /// own work, which keeps Select in hand after the ⌘ hold.
  Down { took: bool },
  /// The press was let go or cancelled.
  Up,
  /// The pointer came onto the picture from the page around it. A platform
  /// whose page never hears the pointer leave (WebKit under the macOS
  /// picture) sends it, so the page can stop showing what it last saw
  /// under the pointer as hovered.
  Arrived,
}

/// A right press on a video layer or an annotation in the native canvas: the
/// pane it landed on, the annotation's place in the published list when it
/// landed on one, and the point, in logical px from the top-left of the
/// window's content, for the web layer to open its own menu at.
pub type ContextMenuCallback = Box<dyn FnMut(u32, Option<u32>, f64, f64) + Send + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionGesturePhase {
  Begin = 0,
  Update = 1,
  End = 2,
  Cancel = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SelectionGestureOperation {
  Move = 0,
  Resize = 1,
  FrameResize = 2,
  CropMove = 3,
  CropResize = 4,
  /// A new crop window drawn from a press outside the current one. Begin
  /// carries the anchor's offset from the crop's origin; later samples carry
  /// the far corner's offset from the anchor, or the half extents when the
  /// centered edge bit is set.
  CropDraw = 5,
  /// A ring of the rotation gizmo dragged round. Every sample carries the
  /// turn it reaches, in degrees: about X in the X delta, about Y in the Y
  /// delta and about Z in the scale.
  Rotate = 6,
}

impl SelectionGestureOperation {
  /// Crop gestures are mirrored by React's uncropped composition rather than
  /// the native gesture managers.
  pub const fn is_crop(self) -> bool {
    matches!(self, Self::CropMove | Self::CropResize | Self::CropDraw)
  }
}

pub type SelectionGestureCallback = Box<
  dyn FnMut(SelectionGesturePhase, u32, SelectionGestureOperation, u32, f64, f64, f64)
    + Send
    + 'static,
>;

/// One pointer sample of an annotation tool: the phase, the pane, whether the
/// gesture makes a new annotation (0) or edits an existing one (1), that
/// annotation's index and grip, the point in the layer's image-normalised
/// space, which snapping modifiers were held - bit 0 Shift, which snaps a
/// counter's tail to the quarter turns, bit 1 Cmd or Ctrl, which snaps the
/// position itself - and how wide the layer's picture is drawn on screen, in
/// points, which is what turns a snap's reach into source pixels.
pub type AnnotationGestureCallback =
  Box<dyn FnMut(SelectionGesturePhase, u32, u32, u32, u32, f64, f64, u32, f64) + Send + 'static>;

/// The hover halo's progress: the arrow the pointer rests on (or -1), how far
/// through the pulse it is, and how wide the layer's picture is drawn on
/// screen, in points.
pub type AnnotationHoverCallback = Box<dyn FnMut(i32, f64, f64) + Send + 'static>;

/// What a text box being typed into reports: a double-click asking to open
/// it, a change to its text, or the end of the typing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationTextPhase {
  Open,
  Change,
  End,
}

/// One report from a text box being typed into: the phase, the layer and the
/// box's place in its list, the text as it now stands, and the change's
/// revision, which only ever grows within one session.
pub type AnnotationTextCallback =
  Box<dyn FnMut(AnnotationTextPhase, u32, u32, String, u64) + Send + 'static>;

/// [`PreviewSelection::framed`] for a rectangle a scene fixes: a move pans
/// the picture inside it, a resize zooms it, and the outline stays put.
pub const FRAMED_FIXED: u32 = 1;
/// [`PreviewSelection::framed`] for a rectangle a scene places but leaves
/// free, a custom scene's box: it moves and resizes like any layer but never
/// grows the canvas, whose size the scene does not own.
pub const FRAMED_FREE: u32 = 2;

/// A render-only selection overlay in workspace coordinates. The rectangle is
/// normalized to its pane so the native surface can apply the same pan/zoom
/// transform as the media without knowing frontend output sizes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PreviewSelection {
  pub pane_index: u32,
  /// Logical editable layer. This is independent of the physical pane: a
  /// baked camera and its screen both render in pane 0 but remain two layers.
  pub layer_id: u32,
  /// Non-zero when this rectangle is a crop window over an independent image.
  pub crop_mode: u32,
  /// Non-zero when a crop window keeps its shape as its handles are pulled,
  /// as a crop given a ratio of width to height does. Such a crop is only
  /// resized and moved: a press outside it draws no new one, which would be
  /// free of any shape.
  pub crop_keeps_shape: u32,
  /// Where a scene places this rectangle: zero outside every scene,
  /// [`FRAMED_FIXED`] where the scene fixes it, and [`FRAMED_FREE`] where the
  /// scene places it but leaves it free to move and resize.
  pub framed: u32,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub radius_percent: f64,
  pub image_x: f64,
  pub image_y: f64,
  pub image_width: f64,
  pub image_height: f64,
  /// The uncropped picture this rectangle is cut from, in pane fractions.
  /// Only crop selections carry it; it bounds the crop magnifier.
  pub recenter_x: f64,
  pub recenter_y: f64,
  pub recenter_width: f64,
  pub recenter_height: f64,
  /// Non-zero where a scene turns this layer in depth and the select tool
  /// turns it with the rotation gizmo's rings: the screen inside a scene.
  pub turnable: u32,
  /// How the layer is turned now, in degrees about each axis, which its
  /// chrome is drawn and pressed through.
  pub rotation_x: f64,
  pub rotation_y: f64,
  pub rotation_z: f64,
  /// Non-zero for a blank canvas's own layer: its annotations are drawn on
  /// and a drawing press on empty canvas joins it, but it is never pressed
  /// as a picture and draws no frame or grips.
  pub blank: u32,
  /// How far the crop's chrome is from shown while the crop opens or
  /// closes: at zero the crop's, its shade and its ants, and at one the
  /// select tool's, a solid outline and, over a turned layer, the gizmo.
  pub crop_closing: f64,
}

impl PreviewSelection {
  /// This selection as one the select tool turns with the rotation gizmo,
  /// turned as `rotation` has it now; `None` leaves it a layer nothing turns.
  pub fn turned_by(self, rotation: Option<screenwide_scenes::SceneRotation>) -> Self {
    Self {
      turnable: u32::from(rotation.is_some()),
      rotation_x: rotation.map_or(0.0, |rotation| rotation.x),
      rotation_y: rotation.map_or(0.0, |rotation| rotation.y),
      rotation_z: rotation.map_or(0.0, |rotation| rotation.z),
      ..self
    }
  }
}
