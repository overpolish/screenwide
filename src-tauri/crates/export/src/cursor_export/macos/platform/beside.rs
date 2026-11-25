// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A format drawn beside the loop's own from each decoded frame. It has
//! buffers of its own to be drawn into and an x264 run of its own; each frame
//! drawn is read back and written to x264 on a thread of its own, so the loop
//! goes on decoding and drawing meanwhile.

use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use cidre::cv;

use super::mux::X264Plan;
use super::*;

mod read_back;
use read_back::{nv12_buffer, write_frames, Buffer, Written};

/// Buffers each format is drawn into: the frames the loop keeps on the GPU,
/// the one it composes before draining the oldest, and room for the writer
/// to fall a few frames behind before the loop waits for it.
const BUFFERS: usize = 8;

struct Writer {
  frames: Sender<Written>,
  thread: JoinHandle<Result<(), String>>,
}

pub(super) struct Beside {
  destination: PathBuf,
  width: usize,
  plan: Option<X264Plan>,
  stream: Option<media_preview::StreamedExport>,
  buffers: Arc<[Buffer]>,
  free: Receiver<usize>,
  /// Handed to the writer once x264 starts; the writer gives buffers back.
  free_sender: Option<Sender<usize>>,
  writer: Option<Writer>,
  /// Frames drawn and not yet written, by the loop's token, oldest first.
  in_flight: VecDeque<(u64, usize)>,
  last_token: Option<u64>,
}

impl Beside {
  pub(super) fn new(request: &CursorExportRequest<'_>) -> Result<Self, String> {
    let (width, height) = (request.output.width, request.output.height);
    let buffers = (0..BUFFERS)
      .map(|_| nv12_buffer(width, height))
      .collect::<Result<Vec<_>, _>>()?;
    let (free_sender, free) = channel();
    for index in 0..BUFFERS {
      let _ = free_sender.send(index);
    }
    Ok(Self {
      destination: request.destination.to_path_buf(),
      width: width as usize,
      plan: Some(export_plan(request)),
      stream: None,
      buffers: buffers.into(),
      free,
      free_sender: Some(free_sender),
      writer: None,
      in_flight: VecDeque::new(),
      last_token: None,
    })
  }

  /// The file this format is written to, which its errors name.
  fn named(&self, error: impl std::fmt::Display) -> String {
    let name = self.destination.file_name().map_or_else(
      || self.destination.display().to_string(),
      |name| name.to_string_lossy().into_owned(),
    );
    format!("{name}: {error}")
  }

  /// Starts x264 at `frame_rate`, and the thread that writes it each frame.
  pub(super) fn start(&mut self, frame_rate: u32) -> Result<(), String> {
    let plan = self
      .plan
      .take()
      .ok_or_else(|| self.named("the encoder was already started"))?;
    let mut stream = plan.start(frame_rate).map_err(|error| self.named(error))?;
    let input = stream
      .take_input()
      .ok_or_else(|| self.named("the encoder has no input"))?;
    let free = self
      .free_sender
      .take()
      .ok_or_else(|| self.named("the encoder was already started"))?;
    let (frames, written) = channel();
    let buffers = Arc::clone(&self.buffers);
    let width = self.width;
    let thread = std::thread::Builder::new()
      .name("screenwide-export-format".into())
      .spawn(move || write_frames(input, &buffers, width, &written, &free))
      .map_err(|error| self.named(error))?;
    self.stream = Some(stream);
    self.writer = Some(Writer { frames, thread });
    Ok(())
  }

  /// A buffer for the frame about to be drawn, waiting while the writer is
  /// behind; `drawn` gives the frame to the writer once the GPU is done.
  pub(super) fn next_buffer(&mut self) -> Result<usize, String> {
    match self.free.recv() {
      Ok(index) => Ok(index),
      Err(_) => Err(self.writer_error()),
    }
  }

  pub(super) fn buffer(&self, index: usize) -> &cv::PixelBuf {
    &self.buffers[index].0
  }

  /// The frame drawn into buffer `index` was submitted as `token`.
  pub(super) fn composed(&mut self, token: u64, index: usize) {
    self.in_flight.push_back((token, index));
  }

  /// The GPU has finished the frame `token`, and every one before it: they
  /// go to the writer in order. A token seen already is the frame shown
  /// again.
  pub(super) fn drawn(&mut self, token: u64) -> Result<(), String> {
    let Some(writer) = self.writer.as_ref() else {
      return Err(self.named("the encoder was never started"));
    };
    let mut sent = false;
    while let Some(&(queued, index)) = self.in_flight.front() {
      if queued > token {
        break;
      }
      self.in_flight.pop_front();
      if writer.frames.send(Written::Frame(index)).is_err() {
        return Err(self.writer_error());
      }
      self.last_token = Some(queued);
      sent = true;
    }
    if !sent && self.last_token == Some(token) && writer.frames.send(Written::Again).is_err() {
      return Err(self.writer_error());
    }
    Ok(())
  }

  /// Why the writer stopped, which it says as it ends.
  fn writer_error(&mut self) -> String {
    let stopped = self.writer.take().map(|writer| {
      drop(writer.frames);
      writer.thread.join()
    });
    let encoder = self
      .stream
      .take()
      .and_then(media_preview::StreamedExport::abandon);
    let error = match stopped {
      Some(Ok(Err(error))) => encoder.unwrap_or(error),
      _ => encoder.unwrap_or_else(|| "the encoder stopped taking frames".to_owned()),
    };
    self.named(error)
  }

  /// Ends the video once every frame is written, and lets x264 finish the
  /// file at its destination.
  pub(super) fn finish(mut self, cancelled: &AtomicBool) -> Result<ExportRunResult, String> {
    let writer = self
      .writer
      .take()
      .ok_or_else(|| self.named("the encoder was never started"))?;
    drop(writer.frames);
    match writer.thread.join() {
      Ok(Ok(())) => {}
      Ok(Err(error)) => {
        let encoder = self
          .stream
          .take()
          .and_then(media_preview::StreamedExport::abandon);
        return Err(self.named(encoder.unwrap_or(error)));
      }
      Err(_) => return Err(self.named("the frame writer failed")),
    }
    let stream = self
      .stream
      .take()
      .ok_or_else(|| self.named("the encoder was never started"))?;
    stream
      .finish(&self.destination, cancelled)
      .map_err(|error| self.named(error))
  }

  pub(super) fn destination(&self) -> &Path {
    &self.destination
  }
}

/// A format not finished is stopped, and what its encoder wrote removed.
impl Drop for Beside {
  fn drop(&mut self) {
    // The encoder goes first: a writer blocked on its input then fails
    // rather than waiting on it.
    if let Some(stream) = self.stream.take() {
      let _ = stream.abandon();
    }
    if let Some(writer) = self.writer.take() {
      drop(writer.frames);
      let _ = writer.thread.join();
    }
  }
}
