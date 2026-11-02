// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Ruler and OCR at work in a preview: the tool's document driven by the
//! page's pointer and drawn through the overlay's own region surface, then
//! every display interval while a transition runs, the way the overlay's
//! timers retime it.

use std::time::Duration;

use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_osc::protocol::InputPhase;
use screenwide_settings_preview::scene::PreviewInput;
use screenwide_settings_preview::Renderer;

use crate::app_windows::screenshot_region::region_osc::preview::PreviewSurface;
use crate::ruler::preview::RulerPreview;
use crate::text_recognition::preview::{OcrFrame, OcrPreview};

/// About one display interval.
const FRAME: Duration = Duration::from_millis(16);

#[expect(
  clippy::large_enum_variant,
  reason = "one driver lives on each preview's thread"
)]
enum Driver {
  Ruler(RulerPreview),
  TextRecognition(OcrPreview),
}

pub(super) struct ToolRenderer {
  surface: PreviewSurface,
  driver: Driver,
  ruler_animating: bool,
}

impl ToolRenderer {
  /// The Ruler where `ruler`, otherwise OCR, on `picture`, `size` points in
  /// size, drawn on `drawable`.
  pub(super) fn new(
    ruler: bool,
    picture: CapturedImage,
    size: Size,
    drawable: screenwide_gpu::surface::Surface,
  ) -> Result<Self, String> {
    let pixels = (picture.width, picture.height);
    let scale = f64::from(picture.width) / size.width.max(1.0);
    let surface = PreviewSurface::new(drawable, pixels, scale)?;
    Ok(Self::on(surface, ruler, picture, size))
  }

  /// The tool on `surface`, set up as `new` does.
  fn on(mut surface: PreviewSurface, ruler: bool, picture: CapturedImage, size: Size) -> Self {
    surface.set_picture(&picture);
    let driver = if ruler {
      surface.configure_ruler(Rect::from_xywh(0.0, 0.0, size.width, size.height));
      Driver::Ruler(RulerPreview::new(picture, size))
    } else {
      surface.configure_text_recognition();
      Driver::TextRecognition(OcrPreview::new(picture, size))
    };
    surface.draw();
    Self {
      surface,
      driver,
      ruler_animating: false,
    }
  }

  /// Another frame is due while a transition runs or the recognizer reads.
  fn waiting(&self) -> Option<Duration> {
    let reading = matches!(&self.driver, Driver::TextRecognition(ocr) if ocr.is_reading());
    (self.ruler_animating || self.surface.is_animating() || reading).then_some(FRAME)
  }

  fn reset(&mut self) {
    match &mut self.driver {
      // The next input draws the cleared document.
      Driver::Ruler(ruler) => ruler.reset(),
      Driver::TextRecognition(ocr) => {
        let frames = ocr.reset();
        show(&mut self.surface, frames);
      }
    }
    self.ruler_animating = false;
  }
}

impl Renderer for ToolRenderer {
  fn input(&mut self, input: PreviewInput, x: f64, y: f64) -> Option<Duration> {
    let phase = match input {
      PreviewInput::Hover => InputPhase::Hover,
      PreviewInput::Down => InputPhase::Down,
      PreviewInput::Drag => InputPhase::Drag,
      PreviewInput::Up => InputPhase::Up,
      PreviewInput::Reset => {
        self.reset();
        return self.waiting();
      }
      PreviewInput::PickArrow
      | PreviewInput::PickCounter
      | PreviewInput::Exit
      | PreviewInput::Enter => return self.waiting(),
    };
    let point = Point { x, y };
    match &mut self.driver {
      Driver::Ruler(ruler) => {
        if let Some(result) = ruler.input(phase, point) {
          self.ruler_animating = self.surface.apply_ruler(ruler.state(), &result);
        }
      }
      Driver::TextRecognition(ocr) => {
        let frames = ocr.input(phase, point);
        show(&mut self.surface, frames);
      }
    }
    self.waiting()
  }

  /// The Ruler's next animation frame, or the recognizer's answer.
  fn frame(&mut self) -> Option<Duration> {
    match &mut self.driver {
      Driver::Ruler(ruler) if self.ruler_animating => {
        self.ruler_animating = ruler
          .input(InputPhase::RulerAnimationFrame, Point::default())
          .is_some_and(|result| self.surface.apply_ruler(ruler.state(), &result));
      }
      Driver::Ruler(_) => {}
      Driver::TextRecognition(ocr) => {
        if let Some(packet) = ocr.poll() {
          self.surface.apply_ocr(&packet);
        }
      }
    }
    if self.surface.is_animating() {
      self.surface.draw();
    }
    self.waiting()
  }
}

fn show(surface: &mut PreviewSurface, frames: Vec<OcrFrame>) {
  for frame in frames {
    match frame {
      OcrFrame::Region(region) => surface.set_region(region),
      OcrFrame::Packet(packet) => surface.apply_ocr(&packet),
    }
  }
}

#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;
