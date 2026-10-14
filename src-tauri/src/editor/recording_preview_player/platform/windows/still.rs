// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Long-lived Media Foundation decoder for paused frames and timeline scrubs.

use std::{
  sync::{atomic::Ordering, mpsc},
  thread::JoinHandle,
};

use tauri::ipc::Channel;

use self::seeking::{frame_is_stale, reuses_settled, should_seek};
use super::present_native_frames;
use crate::editor::recording_preview_player::{PlayerSources, RecordingPreviewPlayerEvent};
use screenwide_export::decode::windows::gpu_decoder::{GpuFrame, GpuVideoReader};

mod seeking;

enum DecoderCommand {
  Seek {
    position_ms: u64,
    request_id: u64,
    rough: bool,
  },
  Stop,
}

pub(crate) struct NativeStillDecoder {
  sender: mpsc::Sender<DecoderCommand>,
  thread: Option<JoinHandle<()>>,
}

impl NativeStillDecoder {
  pub(crate) fn spawn(
    sources: PlayerSources,
    event_channel: Channel<RecordingPreviewPlayerEvent>,
  ) -> Result<Self, String> {
    let (sender, receiver) = mpsc::channel();
    let thread = std::thread::Builder::new()
      .name("recording-preview-still-windows".to_owned())
      .spawn(move || run(sources, receiver, event_channel))
      .map_err(|error| error.to_string())?;
    Ok(Self {
      sender,
      thread: Some(thread),
    })
  }

  pub(crate) fn seek(
    &self,
    position_ms: u64,
    request_id: u64,
    rough: bool,
    _target_sizes: Vec<(u32, u32)>,
  ) -> Result<(), String> {
    self
      .sender
      .send(DecoderCommand::Seek {
        position_ms,
        request_id,
        rough,
      })
      .map_err(|_| "The Windows preview decoder is no longer running".to_owned())
  }

  pub(crate) fn stop(mut self) {
    let _ = self.sender.send(DecoderCommand::Stop);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }

  pub(crate) fn is_finished(&self) -> bool {
    self
      .thread
      .as_ref()
      .is_some_and(std::thread::JoinHandle::is_finished)
  }
}

fn run(
  sources: PlayerSources,
  receiver: mpsc::Receiver<DecoderCommand>,
  event_channel: Channel<RecordingPreviewPlayerEvent>,
) {
  let Some(surface) = sources.preview_surface.clone() else {
    let _ = event_channel.send(RecordingPreviewPlayerEvent::Error {
      message: "Windows GPU preview has no native presentation surface".to_owned(),
    });
    return;
  };
  let mut paths = vec![sources.screen_path.clone()];
  if let Some(path) = &sources.camera_path {
    paths.push(path.clone());
  }
  let mut readers = (0..paths.len()).map(|_| None).collect::<Vec<_>>();
  let mut pending = None;
  // The frames the last settled seek presented, and the position they were
  // decoded for. A settled seek reopens every reader, which costs a Media
  // Foundation open and a decode from the keyframe; a caption, cursor or
  // annotation change asks for the same position again, so it presents these
  // with the current settings instead. Each reader still holds the sample
  // behind its frame, since nothing reads from it until a seek elsewhere,
  // which drops these first.
  let mut settled: Option<(u64, [Option<GpuFrame>; 2])> = None;
  while let Ok(mut command) = pending.take().map_or_else(|| receiver.recv(), Ok) {
    while let Ok(next) = receiver.try_recv() {
      command = next;
    }
    let DecoderCommand::Seek {
      position_ms,
      request_id,
      rough,
    } = command
    else {
      break;
    };
    if let Some((_, [screen, camera])) = settled
      .as_ref()
      .filter(|(settled_ms, _)| reuses_settled(Some(*settled_ms), position_ms, rough))
    {
      if sources.playing.load(Ordering::Acquire) {
        continue;
      }
      let presented = present_native_frames(&sources, screen.as_ref(), camera.as_ref(), 0.0);
      if presented {
        announce_ready(&sources, &event_channel, position_ms, request_id);
        continue;
      }
    }
    settled = None;
    let mut presented = true;
    let mut frames = [None, None];
    for (index, (path, reader)) in paths.iter().zip(readers.iter_mut()).enumerate() {
      // Repeated SetCurrentPosition calls can leave Media Foundation's D3D
      // source reader at a premature EOF even though later samples exist.
      // A settled seek must be authoritative, so give it a fresh native GPU
      // reader, which rewinds only when a seek without one misses. Rough
      // seeks keep their warm reader unless it demonstrably falls behind,
      // preserving fast drag feedback.
      if reader.is_none() || !rough {
        match GpuVideoReader::open_at(path, position_ms, surface.device()) {
          Ok(opened) => *reader = Some(opened),
          Err(message) => {
            let _ = event_channel.send(RecordingPreviewPlayerEvent::Error { message });
            presented = false;
            break;
          }
        }
      } else if let Some(reader) = reader.as_mut() {
        if should_seek(reader.last_timestamp_ms(), position_ms, rough) {
          if let Err(message) = reader.seek(position_ms, rough) {
            let _ = event_channel.send(RecordingPreviewPlayerEvent::Error { message });
            presented = false;
            break;
          }
        }
      }
      let decoded = match reader
        .as_mut()
        .and_then(|reader| reader.frame_at(position_ms).transpose())
      {
        Some(Ok(frame)) => Some(frame),
        Some(Err(message)) => {
          let _ = event_channel.send(RecordingPreviewPlayerEvent::Error { message });
          presented = false;
          break;
        }
        None => None,
      };
      let needs_reopen = decoded
        .as_ref()
        .is_none_or(|frame| frame_is_stale(frame.timestamp_ms, position_ms));
      let frame = if needs_reopen && rough {
        // Reopening a Media Foundation reader costs far more than the drag has
        // to spare. Retry the seek on the warm reader instead and, if it still
        // has nothing at this position, show the pane's previous frame for one
        // more step; the settled seek that ends the drag is authoritative.
        let recovered = reader.as_mut().and_then(|reader| {
          reader
            .seek(position_ms, true)
            .and_then(|()| reader.frame_at(position_ms))
            .ok()
            .flatten()
        });
        let Some(recovered) = recovered else {
          presented = false;
          break;
        };
        recovered
      } else if needs_reopen {
        match GpuVideoReader::open(path, position_ms, surface.device()).and_then(|mut opened| {
          let frame = opened.frame_at(position_ms)?;
          *reader = Some(opened);
          frame.ok_or_else(|| "Media Foundation returned no frame after reopening".to_owned())
        }) {
          Ok(recovered) => recovered,
          Err(message) => {
            let _ = event_channel.send(RecordingPreviewPlayerEvent::Error { message });
            presented = false;
            break;
          }
        }
      } else {
        decoded.expect("a non-stale decoded frame exists")
      };
      if sources.playing.load(Ordering::Acquire) {
        presented = false;
        frames = [None, None];
        break;
      }
      if let Some(slot) = frames.get_mut(index) {
        *slot = Some(frame);
      }
    }
    // A pane that failed still shows the others' frames. A paused still is
    // not moving, so its annotations are not blurred.
    let [screen, camera] = &frames;
    if screen.is_some() || camera.is_some() {
      presented &= present_native_frames(&sources, screen.as_ref(), camera.as_ref(), 0.0);
    }
    if presented {
      announce_ready(&sources, &event_channel, position_ms, request_id);
      if !rough {
        settled = Some((position_ms, frames));
      }
    }
  }
}

/// Marks the objects picked at `position_ms` and tells the editor the still
/// for `request_id` is on screen.
fn announce_ready(
  sources: &PlayerSources,
  event_channel: &Channel<RecordingPreviewPlayerEvent>,
  position_ms: u64,
  request_id: u64,
) {
  let camera_ms =
    crate::editor::recording_preview_player::camera_picking::camera_ms(sources, position_ms);
  sources.let_go_elsewhere(camera_ms);
  sources.show_picked(Some(camera_ms));
  let _ = event_channel.send(RecordingPreviewPlayerEvent::Ready {
    position_ms,
    request_id,
  });
}
