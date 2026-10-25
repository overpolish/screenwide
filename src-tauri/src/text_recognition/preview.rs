// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! OCR at work on a picture rather than the frozen desktop, for a window
//! that shows what OCR does: an area chosen with the pointer, read by the
//! platform's recognizer, and its text selected. It keeps a session of its
//! own, apart from the one the shortcut starts.

use std::sync::mpsc::{self, Receiver, TryRecvError};

use screenwide_imaging::CapturedImage;
use screenwide_osc::desktop::DesktopDisplay;
use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_osc::protocol::{InputModifiers, InputPhase};

use super::input::text_action;
use super::visual::RenderPacket;
use super::{recognize, TextRecognitionResult, TextRecognitionState};
use crate::app_windows::screenshot_region::region_osc::preview::PREVIEW_DISPLAY_ID as DISPLAY_ID;

type Recognized = Result<TextRecognitionResult, String>;

/// What the surface is asked to show after an input.
pub(crate) enum OcrFrame {
  /// The area being chosen, in the picture's points; empty for none.
  Region(Rect),
  Packet(RenderPacket),
}

enum Stage {
  /// The pointer chooses an area, from where it was pressed.
  Choosing(Option<Point>),
  /// The area is with the recognizer, under this generation.
  Reading(u64, Receiver<Recognized>),
  /// The area has been read; the pointer selects its text.
  Read,
}

pub(crate) struct OcrPreview {
  state: TextRecognitionState,
  display: DesktopDisplay,
  picture: CapturedImage,
  stage: Stage,
}

impl OcrPreview {
  /// Reads `picture`, shown at `size` points.
  pub(crate) fn new(picture: CapturedImage, size: Size) -> Self {
    let display = DesktopDisplay {
      id: DISPLAY_ID,
      origin: Point::default(),
      size,
      scale: f64::from(picture.width) / size.width.max(1.0),
    };
    let preview = Self {
      state: TextRecognitionState::default(),
      display,
      picture,
      stage: Stage::Choosing(None),
    };
    preview.install();
    preview
  }

  fn install(&self) {
    let generation = self.state.begin();
    self.state.install(
      generation,
      [(DISPLAY_ID, self.display.scale, self.picture.clone())],
    );
  }

  /// Whether the recognizer has the area, so the answer is polled for.
  pub(crate) fn is_reading(&self) -> bool {
    matches!(self.stage, Stage::Reading(..))
  }

  /// Back to no area chosen, as a new session starts.
  pub(crate) fn reset(&mut self) -> Vec<OcrFrame> {
    self.stage = Stage::Choosing(None);
    self.install();
    vec![
      OcrFrame::Region(Rect::default()),
      OcrFrame::Packet(RenderPacket::default()),
    ]
  }

  /// Applies `phase` at `point`, in the picture's points. Until an area has
  /// been read the pointer chooses one; after, it selects text in it.
  pub(crate) fn input(&mut self, phase: InputPhase, point: Point) -> Vec<OcrFrame> {
    let Stage::Choosing(start) = self.stage else {
      return self.text_input(phase, point);
    };
    match (phase, start) {
      (InputPhase::Down, _) => {
        self.stage = Stage::Choosing(Some(point));
        vec![
          OcrFrame::Packet(RenderPacket::default()),
          OcrFrame::Region(Rect::from_xywh(point.x, point.y, 0.0, 0.0)),
        ]
      }
      (InputPhase::Drag, Some(start)) => vec![OcrFrame::Region(ordered(start, point))],
      (InputPhase::Up, Some(start)) => self.read(ordered(start, point)),
      _ => Vec::new(),
    }
  }

  /// Crops the area from the picture and hands it to the recognizer on a
  /// thread of its own, showing the loading pill until [`Self::poll`] sees
  /// the result.
  fn read(&mut self, region: Rect) -> Vec<OcrFrame> {
    self.stage = Stage::Choosing(None);
    let selected = self
      .state
      .select_desktop_region(&[self.display], DISPLAY_ID, region);
    let Some((generation, image)) = selected.ok().and(self.state.recognition_input()) else {
      return vec![OcrFrame::Region(Rect::default())];
    };
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
      let read = tauri::async_runtime::block_on(recognize(image.rgba, image.width, image.height));
      let _ = sender.send(read.map(|(lines, qr_codes)| {
        let text = lines
          .iter()
          .map(|line| line.text.as_str())
          .collect::<Vec<_>>()
          .join("\n");
        TextRecognitionResult {
          lines,
          qr_codes,
          text,
        }
      }));
    });
    self.stage = Stage::Reading(generation, receiver);
    vec![
      OcrFrame::Region(region),
      OcrFrame::Packet(RenderPacket::loading(screenwide_i18n::t!(
        "overlay-ocr-finding"
      ))),
    ]
  }

  /// The recognizer's answer, once it has one.
  pub(crate) fn poll(&mut self) -> Option<RenderPacket> {
    let Stage::Reading(generation, receiver) = &self.stage else {
      return None;
    };
    let generation = *generation;
    let read = match receiver.try_recv() {
      Ok(read) => read,
      Err(TryRecvError::Empty) => return None,
      Err(TryRecvError::Disconnected) => Err("The recognizer stopped".to_owned()),
    };
    let snapshot = read.and_then(|result| {
      self
        .state
        .install_result(generation, result)
        .then(|| self.state.visual_snapshot(generation))
        .flatten()
        .ok_or_else(|| "The read area is no longer current".to_owned())
    });
    Some(match snapshot {
      Ok(snapshot) => {
        self.stage = Stage::Read;
        RenderPacket::ready(&snapshot)
      }
      Err(error) => {
        self.stage = Stage::Choosing(None);
        RenderPacket::error(error)
      }
    })
  }

  fn text_input(&mut self, phase: InputPhase, point: Point) -> Vec<OcrFrame> {
    let Some(action) = text_action(phase, InputModifiers::default()) else {
      return Vec::new();
    };
    self
      .state
      .text_input(action, point)
      .and_then(|update| update.snapshot)
      .map(|snapshot| vec![OcrFrame::Packet(RenderPacket::ready(&snapshot))])
      .unwrap_or_default()
  }
}

fn ordered(start: Point, end: Point) -> Rect {
  Rect::from_xywh(
    start.x.min(end.x),
    start.y.min(end.y),
    (start.x - end.x).abs(),
    (start.y - end.y).abs(),
  )
}
