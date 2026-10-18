// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The selection chrome laid over the workspace: built here as vertices of
//! the shared OSC shader, and drawn by [`ChromeRenderer`] into a drawable the
//! platform provides.

mod annotation;
mod drawing;
mod renderer;

pub(crate) use annotation::AnnotationChrome;
pub(crate) use drawing::chrome;
pub use renderer::ChromeRenderer;

use screenwide_osc::{
  geometry::{Point, Rect, Size},
  gpu::vertices::{self as osc_gpu, RenderConstants, Vertex},
};

fn logical_rect(rect: [f32; 4], scale: f64) -> Rect {
  Rect::from_xywh(
    f64::from(rect[0]) / scale,
    f64::from(rect[1]) / scale,
    f64::from(rect[2]) / scale,
    f64::from(rect[3]) / scale,
  )
}
