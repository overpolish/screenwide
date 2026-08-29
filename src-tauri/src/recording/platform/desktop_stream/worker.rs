// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use cidre::{arc, cv};

use super::super::*;
use crate::desktop_capture::CapturePlan;

use crate::recording::platform::desktop_compositor::DesktopFrameCoordinator;
use crate::recording::platform::output::FrameClock;

pub(super) struct DesktopSourceFrame {
  pub buf: arc::R<cv::PixelBuf>,
  pub source_index: usize,
  pub source_ns: i64,
  pub wall: Instant,
}

pub(super) enum DesktopCompositionMessage {
  Frame(DesktopSourceFrame),
  Stop,
}

// SAFETY: the capture callback retains the buffer and moves that ownership
// into the bounded channel. Only the composition worker reads it afterwards.
unsafe impl Send for DesktopSourceFrame {}

pub(super) struct DesktopCompositionWorker {
  pub sender: Option<SyncSender<DesktopCompositionMessage>>,
  thread: Option<JoinHandle<()>>,
}

impl DesktopCompositionWorker {
  pub fn spawn(
    plan: CapturePlan,
    commands: SyncSender<Command>,
    stats: Arc<CaptureStats>,
  ) -> Result<Self, String> {
    let (sender, receiver) =
      mpsc::sync_channel::<DesktopCompositionMessage>(STREAM_QUEUE_DEPTH as usize);
    let (ready, readiness) = mpsc::channel();
    let thread = std::thread::Builder::new()
      .name("screenwide-desktop-compositor".to_owned())
      .spawn(move || {
        let mut coordinator = match DesktopFrameCoordinator::new(&plan) {
          Ok(coordinator) => {
            let _ = ready.send(Ok(()));
            coordinator
          }
          Err(error) => {
            let _ = ready.send(Err(error));
            return;
          }
        };
        while let Ok(message) = receiver.recv() {
          let DesktopCompositionMessage::Frame(source) = message else {
            break;
          };
          match coordinator.update(source.source_index, source.source_ns, &source.buf) {
            Ok(Some(composed)) => {
              let frame = Frame {
                buf: composed.buffer,
                clock: FrameClock::Source(composed.timestamp_ns),
                wall: source.wall,
              };
              if let Err(TrySendError::Full(_)) = commands.try_send(Command::Frame(frame)) {
                stats.dropped.fetch_add(1, Ordering::Relaxed);
              }
            }
            Ok(None) => {}
            Err(error) => {
              stats.rejected.fetch_add(1, Ordering::Relaxed);
              eprintln!("Desktop frame composition failed: {error}");
              break;
            }
          }
        }
      })
      .map_err(|error| error.to_string())?;
    readiness
      .recv()
      .map_err(|_| "The desktop compositor did not initialize".to_owned())??;
    Ok(Self {
      sender: Some(sender),
      thread: Some(thread),
    })
  }

  pub fn stop(&mut self) {
    if let Some(sender) = self.sender.take() {
      let _ = sender.send(DesktopCompositionMessage::Stop);
    }
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

impl Drop for DesktopCompositionWorker {
  fn drop(&mut self) {
    self.stop();
  }
}
