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
//! point, in a view that reaches at least `SHADOW_ROOM` past the picture on
//! every side.

use screenwide_osc::geometry::{Rect, Size};
use screenwide_osc::gpu::vertices::{self as osc_gpu, RenderConstants, Vertex};

use super::fit::SHADOW_ROOM;

/// The picture, its corners rounded.
const PICTURE_KIND: u32 = 71;
/// The shadow round the picture.
const SHADOW_KIND: u32 = 70;

/// The view round a picture at `picture`, just holding its shadow.
pub(crate) fn view_round(picture: Rect) -> Rect {
  Rect::from_xywh(
    picture.origin.x - SHADOW_ROOM,
    picture.origin.y - SHADOW_ROOM,
    picture.size.width + SHADOW_ROOM * 2.0,
    picture.size.height + SHADOW_ROOM * 2.0,
  )
}

/// The picture at `picture` in a view `view` in size.
pub(crate) fn picture_vertices(view: Size, picture: Rect) -> Vec<Vertex> {
  let mut vertices = Vec::with_capacity(6);
  osc_gpu::add_texture_quad(
    &mut vertices,
    view,
    picture,
    Rect::from_xywh(0.0, 0.0, 1.0, 1.0),
    PICTURE_KIND,
  );
  vertices
}

/// The chrome round the picture at `picture` in a view `view` in size: the
/// shadow, and the selection's frame and handles while `handles`.
pub(crate) fn chrome_vertices(view: Size, picture: Rect, scale: f64, handles: bool) -> Vec<Vertex> {
  let mut vertices = Vec::new();
  // The whole view, its uv running 0 to 1 across the picture and on past it.
  let (width, height) = (picture.size.width.max(1.0), picture.size.height.max(1.0));
  osc_gpu::add_texture_quad(
    &mut vertices,
    view,
    Rect::from_xywh(0.0, 0.0, view.width, view.height),
    Rect::from_xywh(
      -picture.origin.x / width,
      -picture.origin.y / height,
      view.width / width,
      view.height / height,
    ),
    SHADOW_KIND,
  );
  if handles {
    osc_gpu::add_selection(&mut vertices, view, picture, scale);
  }
  vertices
}

/// The constants both layers draw with, at `scale` pixels to the point, for
/// a picture that is `see_through` in places or not.
pub(crate) fn constants(light_appearance: bool, scale: f64, see_through: bool) -> RenderConstants {
  let mut constants = RenderConstants::new(light_appearance);
  constants.reference[0] = scale as f32;
  constants.reference[1] = if see_through { 1.0 } else { 0.0 };
  constants
}
