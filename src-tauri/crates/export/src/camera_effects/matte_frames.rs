// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The matte's frames beside a playing camera. A thread of its own reads
//! them a window at a time, the next window while the last is drawn, so a
//! camera playing on never waits for its matte; a camera moved elsewhere
//! waits for one window read from there.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use super::live::MatteFile;
use crate::decode::LumaReader;
use screenwide_annotations::pin::{FrameSource, LumaFrame};

/// How much of the matte one read takes: a second, so the reader is moved
/// once a second rather than once a frame.
const WINDOW_MS: u64 = 1_000;

type Window = Result<Vec<LumaFrame>, String>;

pub struct MatteFrames {
  file: MatteFile,
  frames: VecDeque<Arc<LumaFrame>>,
  /// Where the frames held end.
  read_to: u64,
  requests: Sender<(u64, u64)>,
  replies: Receiver<Window>,
  /// The window asked for ahead of the camera and not yet taken.
  ahead: Option<(u64, u64)>,
}

impl MatteFrames {
  /// The matte in `file`, read at most `longest_side` pixels on its longer
  /// side.
  pub fn open(file: &MatteFile, longest_side: u32) -> Result<Self, String> {
    let (requests, asked) = mpsc::channel::<(u64, u64)>();
    let (answer, replies) = mpsc::channel::<Window>();
    let (opened, open) = mpsc::channel();
    let (path, duration_ms) = (file.path.clone(), file.duration_ms());
    // The reader is made on the thread that reads with it: the platform's
    // decoder stays on one thread.
    std::thread::Builder::new()
      .name("camera-matte".to_owned())
      .spawn(move || {
        let mut reader = match LumaReader::open(&path, duration_ms, longest_side) {
          Ok(reader) => {
            let _ = opened.send(Ok(()));
            reader
          }
          Err(error) => {
            let _ = opened.send(Err(error));
            return;
          }
        };
        for (from, to) in asked {
          let mut frames = Vec::new();
          let read = reader.read(from, to, &mut |frame| {
            frames.push(frame);
            true
          });
          if answer.send(read.map(|()| frames)).is_err() {
            return;
          }
        }
      })
      .map_err(|error| error.to_string())?;
    open
      .recv()
      .map_err(|_| "The camera's matte could not be opened".to_owned())??;
    Ok(Self {
      file: file.clone(),
      frames: VecDeque::new(),
      read_to: 0,
      requests,
      replies,
      ahead: None,
    })
  }

  pub(crate) fn file(&self) -> &MatteFile {
    &self.file
  }

  /// The matte of the camera frame showing at camera time `ms`.
  pub fn at(&mut self, ms: u64) -> Result<Option<Arc<LumaFrame>>, String> {
    let slot_ms = self.file.matte_ms(ms);
    if self.held(slot_ms).is_none() {
      // The window read ahead is taken where it holds the frame, and let go
      // where the camera moved elsewhere meanwhile.
      if let Some((from, to)) = self.ahead.take() {
        let frames = self.reply()?;
        if (from..to).contains(&slot_ms) {
          self.frames.extend(frames.into_iter().map(Arc::new));
          self.read_to = to;
        }
      }
    }
    if self.held(slot_ms).is_none() {
      let from = slot_ms.saturating_sub(2);
      let to = self.window_end(from);
      self.ask(from, to)?;
      self.frames = self.reply()?.into_iter().map(Arc::new).collect();
      self.read_to = to;
    }
    let frame = self.held(slot_ms).or_else(|| self.frames.front().cloned());
    self.read_on(slot_ms)?;
    Ok(frame)
  }

  /// Lets go of what the camera has passed and, past the middle of the last
  /// window, asks for the next.
  fn read_on(&mut self, slot_ms: u64) -> Result<(), String> {
    while self
      .frames
      .get(1)
      .is_some_and(|next| next.ms + WINDOW_MS < slot_ms)
    {
      self.frames.pop_front();
    }
    if self.ahead.is_none()
      && self.read_to < self.file.duration_ms()
      && slot_ms + WINDOW_MS / 2 >= self.read_to
    {
      let (from, to) = (self.read_to, self.window_end(self.read_to));
      self.ask(from, to)?;
      self.ahead = Some((from, to));
    }
    Ok(())
  }

  fn window_end(&self, from: u64) -> u64 {
    (from + WINDOW_MS).min(self.file.duration_ms().max(from + 1))
  }

  fn ask(&self, from: u64, to: u64) -> Result<(), String> {
    self
      .requests
      .send((from, to))
      .map_err(|_| "The camera's matte reader has stopped".to_owned())
  }

  fn reply(&self) -> Window {
    self
      .replies
      .recv()
      .map_err(|_| "The camera's matte reader has stopped".to_owned())?
  }

  /// The frame held for the slot starting at `slot_ms`, where the frames
  /// held reach it.
  fn held(&self, slot_ms: u64) -> Option<Arc<LumaFrame>> {
    let first = self.frames.front()?;
    if slot_ms + 2 < first.ms || slot_ms >= self.read_to {
      return None;
    }
    self
      .frames
      .iter()
      .rev()
      .find(|frame| frame.ms <= slot_ms + 2)
      .cloned()
  }
}
