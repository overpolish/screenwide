// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A region surface drawn inside another window: the Ruler or OCR at work on
//! a picture, for a window that shows what a tool does. It is the overlay's
//! own surface, scene and chrome, with the window placement left to its
//! owner and its state set here directly instead of through an overlay
//! session.

use std::sync::{Arc, LazyLock};

use screenwide_gpu::surface::Frame;
use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::Rect;
use screenwide_osc::protocol::OscResult;

use super::host::RegionDrawable;
use super::ocr::overlap_area;
use super::state::{apply_ruler_data, ruler_data};
use super::surface::{Gpu, Surface};
use crate::ruler::RulerState;
use crate::text_recognition::visual::RenderPacket;

/// The display a preview's picture is, in the tool's document. It is the
/// only one, and the surface draws a display's live chrome only for its own.
pub(crate) const PREVIEW_DISPLAY_ID: u32 = 1;

/// The pipeline every preview shares, made with the first.
static GPU: LazyLock<Result<Arc<Gpu>, String>> = LazyLock::new(Gpu::new);

fn gpu() -> Result<Arc<Gpu>, String> {
  GPU.clone()
}

struct Drawable(screenwide_gpu::surface::Surface);

impl RegionDrawable for Drawable {
  fn resize(&mut self, gpu: &screenwide_gpu::Gpu, size: (u32, u32)) {
    self.0.resize(gpu, size);
  }

  fn acquire(&self, gpu: &screenwide_gpu::Gpu) -> Result<Frame, String> {
    self.0.acquire(gpu)
  }
}

pub(crate) struct PreviewSurface {
  surface: Surface,
}

impl PreviewSurface {
  /// A surface drawing into `drawable`, `size` physical pixels at `scale`.
  pub(crate) fn new(
    drawable: screenwide_gpu::surface::Surface,
    size: (u32, u32),
    scale: f64,
  ) -> Result<Self, String> {
    let mut surface = Surface::preview(gpu()?, Box::new(Drawable(drawable)), size, scale);
    surface.display_id = PREVIEW_DISPLAY_ID;
    surface.visible = true;
    surface.show_handles = false;
    Ok(Self { surface })
  }

  /// The frozen desktop the tool works on.
  pub(crate) fn set_picture(&mut self, image: &CapturedImage) -> bool {
    self.surface.snapshot_presented = true;
    self
      .surface
      .set_snapshot(&image.rgba, image.width, image.height)
  }

  /// The Ruler's scene: the whole picture is the region, drawn through its
  /// viewport, with no frame.
  pub(crate) fn configure_ruler(&mut self, desktop: Rect) {
    self.surface.snapshot_composited = true;
    self.surface.show_frame = false;
    self.surface.input_enabled = true;
    self.surface.region = desktop;
  }

  /// OCR's scene: the desktop shaded until an area is chosen.
  pub(crate) fn configure_text_recognition(&mut self) {
    self.surface.snapshot_composited = false;
    self.surface.show_frame = true;
    self.surface.input_enabled = true;
    self.surface.region = Rect::default();
  }

  /// Draws a Ruler result from `state` the way the overlay does. Answers
  /// whether a transition is running and needs animation frames.
  pub(crate) fn apply_ruler(&mut self, state: &RulerState, result: &OscResult) -> bool {
    if result.ruler_flags & 1 == 0 {
      return false;
    }
    let data = ruler_data(state);
    apply_ruler_data(vec![&mut self.surface], &data, result).animating
  }

  /// The area being chosen or read, in the picture's points.
  pub(crate) fn set_region(&mut self, region: Rect) {
    self.surface.set_region(region, true);
  }

  /// Draws an OCR packet the way the overlay's `set_ocr` does for its one
  /// surface, with the presentation's frame.
  pub(crate) fn apply_ocr(&mut self, packet: &RenderPacket) {
    let surface = &mut self.surface;
    let bounds = surface.logical_size();
    let offset = surface.desktop_offset();
    let local = surface.local_rect(surface.region);
    let message = packet.message.replace('\0', " ");
    surface
      .ocr
      .apply(packet.phase as u32, &packet.rects, &message, offset, bounds);
    surface.ocr.set_target(overlap_area(local, bounds) > 0.0);
    if let Some(frame) = packet.presentation.frame {
      surface.show_frame = frame;
    }
    surface.draw();
  }

  pub(crate) fn draw(&mut self) {
    self.surface.draw();
  }

  /// Whether a transition on the surface itself, such as a control's, is
  /// running.
  pub(crate) fn is_animating(&self) -> bool {
    self.surface.is_animating()
  }
}
