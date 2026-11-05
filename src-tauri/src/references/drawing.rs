// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a reference draws, through the shared OSC shader, on every platform
//! (`reference.wgsl`): the picture with its corners rounded on a layer of its
//! own, which takes the reference's opacity, and its chrome on a layer over
//! it at full strength. The chrome is the islands' shadow round the picture,
//! outside it only, which lifts the reference off what it lies over, so a
//! picture of an interface laid over that interface still shows where it
//! ends; under the pointer it adds the selection's frame and point handles,
//! which resize it. Every length is in points, at `scale` pixels to the
//! point, in a view that reaches `SHADOW_ROOM` past the picture on every
//! side.

use screenwide_osc::geometry::{Rect, Size};
use screenwide_osc::gpu::vertices::{self as osc_gpu, RenderConstants, Vertex};

use super::fit::SHADOW_ROOM;

/// The picture, its corners rounded.
const PICTURE_KIND: u32 = 71;
/// The shadow round the picture.
const SHADOW_KIND: u32 = 70;

/// The view the reference draws in, for a picture `picture` in size.
pub(crate) fn view_size(picture: Size) -> Size {
  Size {
    width: picture.width + SHADOW_ROOM * 2.0,
    height: picture.height + SHADOW_ROOM * 2.0,
  }
}

/// The picture in its view.
pub(crate) fn picture_in_view(picture: Size) -> Rect {
  Rect::from_xywh(SHADOW_ROOM, SHADOW_ROOM, picture.width, picture.height)
}

pub(crate) fn picture_vertices(picture: Size) -> Vec<Vertex> {
  let mut vertices = Vec::with_capacity(6);
  osc_gpu::add_texture_quad(
    &mut vertices,
    view_size(picture),
    picture_in_view(picture),
    Rect::from_xywh(0.0, 0.0, 1.0, 1.0),
    PICTURE_KIND,
  );
  vertices
}

/// The chrome: the shadow round the picture, and the selection's frame and
/// handles while `handles`.
pub(crate) fn chrome_vertices(picture: Size, scale: f64, handles: bool) -> Vec<Vertex> {
  let view = view_size(picture);
  let mut vertices = Vec::new();
  // The whole view, its uv running 0 to 1 across the picture and on past it.
  let (room_u, room_v) = (
    SHADOW_ROOM / picture.width.max(1.0),
    SHADOW_ROOM / picture.height.max(1.0),
  );
  osc_gpu::add_texture_quad(
    &mut vertices,
    view,
    Rect::from_xywh(0.0, 0.0, view.width, view.height),
    Rect::from_xywh(-room_u, -room_v, 1.0 + room_u * 2.0, 1.0 + room_v * 2.0),
    SHADOW_KIND,
  );
  if handles {
    osc_gpu::add_selection(&mut vertices, view, picture_in_view(picture), scale);
  }
  vertices
}

/// The constants both layers draw with, at `scale` pixels to the point.
pub(crate) fn constants(light_appearance: bool, scale: f64) -> RenderConstants {
  let mut constants = RenderConstants::new(light_appearance);
  constants.reference[0] = scale as f32;
  constants
}
