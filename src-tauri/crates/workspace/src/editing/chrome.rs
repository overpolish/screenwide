// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The selection chrome laid over the workspace: built here as vertices of
//! the shared OSC shader, and drawn by [`ChromeRenderer`] into a drawable the
//! platform provides.

mod annotation;
mod drawing;
mod platform_overlay;
mod renderer;

pub(crate) use annotation::AnnotationChrome;
pub(crate) use drawing::chrome;
pub use platform_overlay::ChromeOverlay;
pub use renderer::ChromeRenderer;

use screenwide_osc::{
  geometry::{Point, Rect, Size},
  gpu::vertices::{self as osc_gpu, RenderConstants, Vertex},
};

/// One draw of the chrome over a viewport `size` physical pixels: one draw
/// of the shared OSC shader, every vertex sharing `constants`, over the
/// platform `overlays` laid over the canvas, back to front: none, one, or
/// two while one fades into the other.
pub struct ChromeFrame<'a> {
  pub size: (u32, u32),
  pub vertices: &'a [Vertex],
  pub constants: &'a RenderConstants,
  pub overlays: &'a [ChromeOverlay],
}

fn logical_rect(rect: [f32; 4], scale: f64) -> Rect {
  Rect::from_xywh(
    f64::from(rect[0]) / scale,
    f64::from(rect[1]) / scale,
    f64::from(rect[2]) / scale,
    f64::from(rect[3]) / scale,
  )
}
