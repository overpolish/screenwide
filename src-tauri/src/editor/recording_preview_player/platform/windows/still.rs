// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows' paused frames and timeline scrubs: Media Foundation readers kept
//! warm between seeks.

use std::{path::PathBuf, sync::atomic::Ordering, sync::Arc};

use tauri::ipc::Channel;

use self::seeking::{frame_is_stale, reuses_settled, should_seek};
use super::present_still;
use crate::editor::preview_platform::RecordingPreviewSurface;
use crate::editor::recording_preview_player::platform::still::{announce_ready, Seek, StillFrames};
use crate::editor::recording_preview_player::{PlayerSources, RecordingPreviewPlayerEvent};
use screenwide_export::decode::windows::gpu_decoder::{GpuFrame, GpuVideoReader};

mod seeking;

pub(in crate::editor::recording_preview_player::platform) struct Stills {
  surface: Arc<RecordingPreviewSurface>,
  paths: Vec<PathBuf>,
  readers: Vec<Option<GpuVideoReader>>,
  /// The frames the last settled seek presented, and the position they were
  /// decoded for. A settled seek reopens every reader, which costs a Media
  /// Foundation open and a decode from the keyframe; a caption, cursor or
  /// annotation change asks for the same position again, so it presents these
  /// with the current settings instead. Each reader still holds the sample
  /// behind its frame, since nothing reads from it until a seek elsewhere,
  /// which drops these first.
  settled: Option<(u64, [Option<GpuFrame>; 2])>,
}

impl StillFrames for Stills {
  fn open(sources: &PlayerSources) -> Result<Self, String> {
    let surface = sources
      .preview_surface
      .clone()
      .ok_or_else(|| "Windows GPU preview has no native presentation surface".to_owned())?;
    let mut paths = vec![sources.screen_path.clone()];
    if let Some(path) = &sources.camera_path {
      paths.push(path.clone());
    }
    Ok(Self {
      surface,
      readers: paths.iter().map(|_| None).collect(),
      paths,
      settled: None,
    })
  }

  fn show(
    &mut self,
    sources: &PlayerSources,
    seek: Seek,
    events: &Channel<RecordingPreviewPlayerEvent>,
  ) {
    let Seek {
      position_ms,
      request_id,
      rough,
      ..
    } = seek;
    if let Some((_, [screen, camera])) = self
      .settled
      .as_ref()
      .filter(|(settled_ms, _)| reuses_settled(Some(*settled_ms), position_ms, rough))
    {
      if sources.playing.load(Ordering::Acquire) {
        return;
      }
      let presented = present_still(sources, screen.as_ref(), camera.as_ref());
      if presented {
        announce_ready(sources, events, (position_ms, position_ms), request_id);
        return;
      }
    }
    self.settled = None;
    let mut presented = true;
    let mut frames = [None, None];
    for (index, (path, reader)) in self.paths.iter().zip(self.readers.iter_mut()).enumerate() {
      // Repeated SetCurrentPosition calls can leave Media Foundation's D3D
      // source reader at a premature EOF even though later samples exist.
      // A settled seek must be authoritative, so give it a fresh native GPU
      // reader, which rewinds only when a seek without one misses. Rough
      // seeks keep their warm reader unless it demonstrably falls behind,
      // preserving fast drag feedback.
      if reader.is_none() || !rough {
        match GpuVideoReader::open_at(path, position_ms, self.surface.device()) {
          Ok(opened) => *reader = Some(opened),
          Err(message) => {
            let _ = events.send(RecordingPreviewPlayerEvent::Error { message });
            presented = false;
            break;
          }
        }
      } else if let Some(reader) = reader.as_mut() {
        if should_seek(reader.last_timestamp_ms(), position_ms, rough) {
          if let Err(message) = reader.seek(position_ms, rough) {
            let _ = events.send(RecordingPreviewPlayerEvent::Error { message });
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
          let _ = events.send(RecordingPreviewPlayerEvent::Error { message });
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
        match GpuVideoReader::open(path, position_ms, self.surface.device()).and_then(
          |mut opened| {
            let frame = opened.frame_at(position_ms)?;
            *reader = Some(opened);
            frame.ok_or_else(|| "Media Foundation returned no frame after reopening".to_owned())
          },
        ) {
          Ok(recovered) => recovered,
          Err(message) => {
            let _ = events.send(RecordingPreviewPlayerEvent::Error { message });
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
      presented &= present_still(sources, screen.as_ref(), camera.as_ref());
    }
    if presented {
      announce_ready(sources, events, (position_ms, position_ms), request_id);
      if !rough {
        self.settled = Some((position_ms, frames));
      }
    }
  }
}
