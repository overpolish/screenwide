// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the live overlay draws on every display, on both platforms: the
//! annotations through the editor's own annotation shader, over what each
//! display's highlights recolour. The platform owns the windows, the layers
//! and the surfaces these frames land in.
//!
//! Nothing is mirrored: every frame pulls [`super::live_clips`] and the stroke
//! in hand, so there is no second copy of the document to keep in step.

mod renderer;
#[cfg(test)]
mod tests;
mod underlay;

pub(super) use renderer::Renderer;
#[cfg(target_os = "windows")]
pub(super) use underlay::upload as upload_underlay;
pub(super) use underlay::Underlays;

use crate::editor::preview_platform::annotation_gpu::{placed_arrows, PreparedArrows};

/// Everything on screen plus the stroke in hand, prepared in one display's
/// layer pixels: `origin` is where the display starts in desktop points and
/// `scale` how many pixels one point is.
///
/// The annotations are carried into the display's pixels first, so the arrows
/// are prepared through the identity placement: the editor's own preparation,
/// with no canvas to fit them to. Their sizes are points, drawn at the
/// display's scale.
fn scene(origin: (f64, f64), scale: f64) -> PreparedArrows {
  let drawn: Vec<_> = super::live_clips::annotations()
    .iter()
    .chain(super::input::in_progress().iter())
    .map(|annotation| super::geometry::display_annotation(annotation, origin, scale))
    .collect();
  placed_arrows(&drawn, (0.0, 0.0), (1.0, 1.0), scale, None, None)
}

impl Renderer {
  /// Draws display `index`'s frame into `target`, `size` pixels, over a
  /// cleared target.
  pub(super) fn draw_display(
    &self,
    target: &wgpu::TextureView,
    size: (u32, u32),
    index: u32,
    origin: (f64, f64),
    scale: f64,
    underlays: &mut Underlays,
  ) -> Result<(), String> {
    let (underlay, softened) = underlays.current(self.gpu(), index)?;
    self.draw_arrows(
      target,
      size,
      &scene(origin, scale),
      underlay.as_ref(),
      softened.as_ref(),
    )
  }
}
