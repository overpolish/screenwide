// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The thread a preview draws on. It owns the renderer and its surface,
//! takes the page's messages in order, and draws whenever the renderer has
//! something due: after a message, then at the times it asks for while
//! something moves.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::scene::{PreviewInput, PreviewScene};

pub enum Message {
  /// The page's pointer, in the picture's points.
  Input {
    input: PreviewInput,
    x: f64,
    y: f64,
  },
  /// The same scene with other values.
  Scene(PreviewScene),
  /// The picture's size on screen, in physical pixels.
  Resize((u32, u32)),
  Visible(bool),
  Stop,
}

/// What draws a preview. Each call answers how long until it next has
/// something to draw, `None` when only a message can change what it shows.
pub trait Renderer {
  fn input(&mut self, _input: PreviewInput, _x: f64, _y: f64) -> Option<Duration> {
    None
  }

  fn scene(&mut self, _scene: PreviewScene) -> Option<Duration> {
    None
  }

  fn resize(&mut self, _size: (u32, u32)) -> Option<Duration> {
    None
  }

  fn visible(&mut self, _visible: bool) -> Option<Duration> {
    None
  }

  /// Draws what is due now.
  fn frame(&mut self) -> Option<Duration>;
}

pub(crate) fn run(mut renderer: Box<dyn Renderer>, messages: &Receiver<Message>) {
  let mut due = renderer.frame().map(|wait| Instant::now() + wait);
  loop {
    let message = match due {
      Some(at) => messages.recv_timeout(at.saturating_duration_since(Instant::now())),
      None => messages.recv().map_err(|_| RecvTimeoutError::Disconnected),
    };
    let next = match message {
      Ok(Message::Stop) | Err(RecvTimeoutError::Disconnected) => return,
      Ok(Message::Input { input, x, y }) => renderer.input(input, x, y),
      Ok(Message::Scene(scene)) => renderer.scene(scene),
      Ok(Message::Resize(size)) => renderer.resize(size),
      Ok(Message::Visible(visible)) => renderer.visible(visible),
      Err(RecvTimeoutError::Timeout) => {
        due = None;
        renderer.frame()
      }
    };
    // A message never puts off a frame already due sooner.
    if let Some(at) = next.map(|wait| Instant::now() + wait) {
      due = Some(due.map_or(at, |due| due.min(at)));
    }
  }
}
