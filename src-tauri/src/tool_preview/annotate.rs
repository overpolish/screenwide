// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotate's preview thread. It draws only when something changes: live
//! annotations are drawn whole, with nothing animating between inputs.

use std::sync::mpsc::Receiver;

use screenwide_annotations::{AnnotationKind, AnnotationPoint};

use super::worker::Message;
use super::PreviewInput;
use crate::annotate::preview::AnnotatePreview;

/// The overlay's pointer phases.
const DOWN: u32 = 0;
const DRAG: u32 = 1;
const UP: u32 = 2;

pub(super) fn run(
  drawable: screenwide_gpu::surface::Surface,
  pixels: (u32, u32),
  scale: f64,
  messages: &Receiver<Message>,
) {
  let mut preview = match AnnotatePreview::new(drawable, pixels, scale) {
    Ok(preview) => preview,
    Err(error) => {
      eprintln!("The Annotate preview could not start: {error}");
      return;
    }
  };
  while let Ok(message) = messages.recv() {
    match message {
      Message::Input(input, point) => {
        let point = AnnotationPoint {
          x: point.x,
          y: point.y,
        };
        match input {
          PreviewInput::Down => preview.pointer(DOWN, point),
          PreviewInput::Drag => preview.pointer(DRAG, point),
          PreviewInput::Up => preview.pointer(UP, point),
          PreviewInput::PickArrow => preview.pick(AnnotationKind::Arrow),
          PreviewInput::PickCounter => preview.pick(AnnotationKind::Counter),
          PreviewInput::Exit => preview.exit(),
          PreviewInput::Enter => preview.enter(),
          PreviewInput::Reset => preview.reset(),
          PreviewInput::Hover => {}
        }
      }
      Message::Look { color, keep } => preview.set_look(color, keep),
      Message::Stop => return,
    }
  }
}
