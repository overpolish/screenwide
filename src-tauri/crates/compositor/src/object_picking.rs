// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects picked in the camera on the preview, as each platform's surface
//! takes them: the press that picks one, and the objects picked in the frame
//! shown, to draw over the camera.

/// A press on the camera while objects are picked, picking the object at
/// `points`, 0 to 1 across and down the camera's picture, a click's one or a
/// painted line's several, to keep or to remove.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectPress {
  pub points: Vec<[f32; 2]>,
  pub keep: bool,
}

pub type ObjectPickCallback = Box<dyn FnMut(ObjectPress) + Send + 'static>;

/// An object picked in the frame the preview shows, 0 to 1 across and down
/// the camera's picture: removed with the room or kept, whether it is the
/// one chosen, and where among the lines' points the line painted over it
/// is, of none where it was clicked. On macOS the twin of
/// `ScreenwidePickedObject` in `recording_preview_object_picking_macos.h`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickedObjectDot {
  pub x: f32,
  pub y: f32,
  pub id: u32,
  pub removed: u32,
  pub chosen: u32,
  pub line_first: u32,
  pub line_points: u32,
}

/// The camera's selection target, the objects picked in the frame shown,
/// and the points of their painted lines.
pub type ObjectsShown<'a> = (u32, &'a [PickedObjectDot], &'a [[f32; 2]]);
