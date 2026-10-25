// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The thread a preview draws on. It owns the tool's document and the
//! surface, takes the page's pointer input in order, and draws a frame after
//! each, then every display interval while a transition runs, the way the
//! overlay's timers retime it.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_osc::protocol::InputPhase;

use super::{PreviewInput, PreviewTool};
use crate::app_windows::screenshot_region::region_osc::preview::PreviewSurface;
use crate::ruler::preview::RulerPreview;
use crate::text_recognition::preview::{OcrFrame, OcrPreview};

/// About one display interval.
const FRAME: Duration = Duration::from_millis(16);

pub(super) enum Message {
  Input(PreviewInput, Point),
  /// Annotate's colour and whether its annotations are kept.
  Look {
    color: String,
    keep: bool,
  },
  Stop,
}

#[expect(
  clippy::large_enum_variant,
  reason = "one driver lives on each preview's thread"
)]
enum Driver {
  Ruler(RulerPreview),
  TextRecognition(OcrPreview),
}

pub(super) struct Setup {
  pub tool: PreviewTool,
  pub picture: CapturedImage,
  /// The picture's size in points.
  pub size: Size,
  pub drawable: screenwide_gpu::surface::Surface,
}

pub(super) fn run(setup: Setup, messages: Receiver<Message>) {
  let Setup {
    tool,
    picture,
    size,
    drawable,
  } = setup;
  let pixels = (picture.width, picture.height);
  let scale = f64::from(picture.width) / size.width.max(1.0);
  if tool == PreviewTool::Annotate {
    // The page paints the screen under it; only the glass is drawn here.
    return super::annotate::run(drawable, pixels, scale, &messages);
  }
  let mut surface = match PreviewSurface::new(drawable, pixels, scale) {
    Ok(surface) => surface,
    Err(error) => {
      eprintln!("A tool preview could not make its surface: {error}");
      return;
    }
  };
  surface.set_picture(&picture);
  // Annotate has returned above, so the tool is one of the two that draw
  // through the region surface.
  let mut driver = if tool == PreviewTool::Ruler {
    surface.configure_ruler(Rect::from_xywh(0.0, 0.0, size.width, size.height));
    Driver::Ruler(RulerPreview::new(picture, size))
  } else {
    surface.configure_text_recognition();
    Driver::TextRecognition(OcrPreview::new(picture, size))
  };
  surface.draw();

  let mut ruler_animating = false;
  loop {
    let waiting = ruler_animating || surface.is_animating() || driver.is_reading();
    let message = if waiting {
      messages.recv_timeout(FRAME)
    } else {
      messages.recv().map_err(|_| RecvTimeoutError::Disconnected)
    };
    match message {
      Ok(Message::Input(input, point)) => {
        ruler_animating = driver.input(&mut surface, input, point, ruler_animating);
      }
      Ok(Message::Look { .. }) => {}
      Ok(Message::Stop) | Err(RecvTimeoutError::Disconnected) => return,
      Err(RecvTimeoutError::Timeout) => {
        ruler_animating = driver.frame(&mut surface, ruler_animating);
        if surface.is_animating() {
          surface.draw();
        }
      }
    }
  }
}

impl Driver {
  fn is_reading(&self) -> bool {
    matches!(self, Self::TextRecognition(ocr) if ocr.is_reading())
  }

  /// Applies one input. Answers whether the Ruler's transitions still run.
  fn input(
    &mut self,
    surface: &mut PreviewSurface,
    input: PreviewInput,
    point: Point,
    ruler_animating: bool,
  ) -> bool {
    let phase = match input {
      PreviewInput::Hover => InputPhase::Hover,
      PreviewInput::Down => InputPhase::Down,
      PreviewInput::Drag => InputPhase::Drag,
      PreviewInput::Up => InputPhase::Up,
      PreviewInput::Reset => return self.reset(surface),
      PreviewInput::PickArrow
      | PreviewInput::PickCounter
      | PreviewInput::Exit
      | PreviewInput::Enter => return ruler_animating,
    };
    match self {
      Self::Ruler(ruler) => match ruler.input(phase, point) {
        Some(result) => surface.apply_ruler(ruler.state(), &result),
        None => ruler_animating,
      },
      Self::TextRecognition(ocr) => {
        let frames = ocr.input(phase, point);
        show(surface, frames);
        false
      }
    }
  }

  fn reset(&mut self, surface: &mut PreviewSurface) -> bool {
    match self {
      // The next input draws the cleared document.
      Self::Ruler(ruler) => ruler.reset(),
      Self::TextRecognition(ocr) => {
        let frames = ocr.reset();
        show(surface, frames);
      }
    }
    false
  }

  /// One display interval with nothing new from the page: the Ruler's next
  /// animation frame, or the recognizer's answer.
  fn frame(&mut self, surface: &mut PreviewSurface, ruler_animating: bool) -> bool {
    match self {
      Self::Ruler(ruler) if ruler_animating => ruler
        .input(InputPhase::RulerAnimationFrame, Point::default())
        .is_some_and(|result| surface.apply_ruler(ruler.state(), &result)),
      Self::Ruler(_) => false,
      Self::TextRecognition(ocr) => {
        if let Some(packet) = ocr.poll() {
          surface.apply_ocr(&packet);
        }
        false
      }
    }
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
