// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The paused-frame and scrub decoder, the same on every platform: a thread
//! of its own that takes seeks, skips straight to the latest when they queue
//! up, and has the platform's [`StillFrames`] decode and present each one.

use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;

use tauri::ipc::Channel;

use super::backend::Stills;
use crate::editor::recording_preview_player::{
  camera_picking, PlayerSources, RecordingPreviewPlayerEvent,
};
use screenwide_annotations::timing::RecordingAnnotationClip;

/// One seek the editor asked for.
pub(in crate::editor::recording_preview_player::platform) struct Seek {
  pub position_ms: u64,
  pub request_id: u64,
  /// A mid-gesture skim: the decoder may land on the cheapest nearby frame
  /// instead of decoding the exact position.
  pub rough: bool,
  /// Each pane's on-screen size, which a still may be composed at.
  #[cfg_attr(target_os = "windows", allow(dead_code))]
  pub target_sizes: Vec<(u32, u32)>,
  /// The annotations as they stood when the seek was asked for. Windows
  /// composes stills at full size from the live annotations instead.
  #[cfg_attr(target_os = "windows", allow(dead_code))]
  pub annotation_clips: Vec<RecordingAnnotationClip>,
}

/// A platform's paused-frame decoding, living on the still thread.
pub(in crate::editor::recording_preview_player::platform) trait StillFrames:
  Sized
{
  fn open(sources: &PlayerSources) -> Result<Self, String>;

  /// Decodes and presents the frame `seek` asks for, and announces it with
  /// [`announce_ready`] once it is on screen.
  fn show(
    &mut self,
    sources: &PlayerSources,
    seek: Seek,
    events: &Channel<RecordingPreviewPlayerEvent>,
  );
}

enum Command {
  Seek(Seek),
  Stop,
}

pub(crate) struct StillDecoder {
  annotation_clips: Arc<RwLock<Vec<RecordingAnnotationClip>>>,
  sender: mpsc::Sender<Command>,
  thread: Option<JoinHandle<()>>,
}

impl StillDecoder {
  pub(crate) fn spawn(
    sources: PlayerSources,
    events: Channel<RecordingPreviewPlayerEvent>,
  ) -> Result<Self, String> {
    let (sender, receiver) = mpsc::channel();
    let annotation_clips = Arc::clone(&sources.annotation_clips);
    let thread = std::thread::Builder::new()
      .name("recording-preview-still".to_owned())
      .spawn(move || run::<Stills>(&sources, &receiver, &events))
      .map_err(|error| error.to_string())?;
    Ok(Self {
      annotation_clips,
      sender,
      thread: Some(thread),
    })
  }

  pub(crate) fn seek(
    &self,
    position_ms: u64,
    request_id: u64,
    rough: bool,
    target_sizes: Vec<(u32, u32)>,
  ) -> Result<(), String> {
    let annotation_clips = self
      .annotation_clips
      .read()
      .map_err(|_| "The annotations are unavailable".to_owned())?
      .clone();
    self
      .sender
      .send(Command::Seek(Seek {
        position_ms,
        request_id,
        rough,
        target_sizes,
        annotation_clips,
      }))
      .map_err(|_| "The preview decoder stopped".to_owned())
  }

  pub(crate) fn stop(mut self) {
    let _ = self.sender.send(Command::Stop);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }

  pub(crate) fn is_finished(&self) -> bool {
    self.thread.as_ref().is_some_and(JoinHandle::is_finished)
  }
}

fn run<W: StillFrames>(
  sources: &PlayerSources,
  receiver: &mpsc::Receiver<Command>,
  events: &Channel<RecordingPreviewPlayerEvent>,
) {
  let mut stills = match W::open(sources) {
    Ok(stills) => stills,
    Err(message) => {
      let _ = events.send(RecordingPreviewPlayerEvent::Error { message });
      return;
    }
  };
  while let Ok(mut command) = receiver.recv() {
    // Only the latest seek matters; those it overtook are never shown.
    while let Ok(next) = receiver.try_recv() {
      command = next;
    }
    let Command::Seek(seek) = command else {
      break;
    };
    stills.show(sources, seek, events);
  }
}

/// Marks the objects picked at `frame_ms`, the moment the still shows, and
/// tells the editor the still for `request_id`, asked for at `position_ms`,
/// is on screen.
pub(in crate::editor::recording_preview_player::platform) fn announce_ready(
  sources: &PlayerSources,
  events: &Channel<RecordingPreviewPlayerEvent>,
  (position_ms, frame_ms): (u64, u64),
  request_id: u64,
) {
  let camera_ms = camera_picking::camera_ms(sources, frame_ms);
  sources.let_go_elsewhere(camera_ms);
  sources.show_picked(Some(camera_ms));
  let _ = events.send(RecordingPreviewPlayerEvent::Ready {
    position_ms,
    request_id,
  });
}

#[cfg(test)]
mod tests;
