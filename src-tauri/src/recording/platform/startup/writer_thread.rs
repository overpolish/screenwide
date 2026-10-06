// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::replay::{ReplayWriter, ReplayWriterConfig};
use super::super::*;

/// The writer thread, once it has confirmed it can write.
pub(super) struct WriterThread {
  pub(super) commands: SyncSender<Command>,
  pub(super) first_frame: Receiver<Result<(), String>>,
  pub(super) worker: JoinHandle<()>,
}

/// A writer that serves one capture's commands until it is told to stop.
trait WriterLoop {
  fn serve(self, inbox: &Receiver<Command>, first_frame: &mpsc::Sender<Result<(), String>>);
}

impl WriterLoop for Writer {
  fn serve(self, inbox: &Receiver<Command>, first_frame: &mpsc::Sender<Result<(), String>>) {
    self.run(inbox, first_frame);
  }
}

impl WriterLoop for ReplayWriter {
  fn serve(self, inbox: &Receiver<Command>, first_frame: &mpsc::Sender<Result<(), String>>) {
    self.run(inbox, first_frame);
  }
}

pub(super) fn spawn_writer(config: WriterConfig, name: &str) -> Result<WriterThread, String> {
  spawn(name, move || Writer::new(config))
}

pub(super) fn spawn_replay_writer(
  config: ReplayWriterConfig,
  name: &str,
) -> Result<WriterThread, String> {
  spawn(name, move || ReplayWriter::new(config))
}

/// Starts a writer thread, building the writer on it: AVFoundation and
/// VideoToolbox objects live and die on the thread that made them.
fn spawn<W: WriterLoop>(
  name: &str,
  build: impl FnOnce() -> Result<W, String> + Send + 'static,
) -> Result<WriterThread, String> {
  let (commands, inbox) = mpsc::sync_channel(FRAME_QUEUE_DEPTH);
  let (ready, readied) = mpsc::channel();
  let (first_frame, first_framed) = mpsc::channel();
  let worker = std::thread::Builder::new()
    .name(name.to_owned())
    .spawn(move || match build() {
      Ok(writer) => {
        let _ = ready.send(Ok(()));
        writer.serve(&inbox, &first_frame);
      }
      Err(error) => {
        let _ = ready.send(Err(error));
      }
    })
    .map_err(|error| error.to_string())?;

  match readied.recv() {
    Ok(Ok(())) => Ok(WriterThread {
      commands,
      first_frame: first_framed,
      worker,
    }),
    Ok(Err(error)) => {
      let _ = worker.join();
      Err(error)
    }
    Err(_) => {
      let _ = worker.join();
      Err("The recording's encoder could not be started".to_owned())
    }
  }
}

pub(super) fn both_first_frames(
  primary: Receiver<Result<(), String>>,
  camera: Receiver<Result<(), String>>,
) -> Receiver<Result<(), String>> {
  let (ready, first_frames) = mpsc::channel();
  std::thread::spawn(move || {
    let result = primary
      .recv()
      .unwrap_or_else(|_| Err("The primary video produced no frames".to_owned()))
      .and_then(|()| {
        camera
          .recv()
          .unwrap_or_else(|_| Err("The camera produced no frames".to_owned()))
      });
    let _ = ready.send(result);
  });
  first_frames
}
