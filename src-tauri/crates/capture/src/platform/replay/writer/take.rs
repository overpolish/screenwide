// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Answering a save: what this writer holds for the clip, on its own thread.

use super::super::{ClipFrom, ClipTake, VideoTake};
use super::*;

impl ReplayWriter {
  pub(super) fn take(&mut self, at: Instant, from: ClipFrom) -> Result<ClipTake, String> {
    if let Some(failure) = &self.failed {
      return Err(failure.clone());
    }
    if !self.timeline.has_started() {
      return Err("The replay buffer has not captured anything yet".to_owned());
    }
    let wall_ns = self.elapsed_ns(at);
    let end_ns = if self.primary_video {
      self.close_at(wall_ns)?
    } else {
      // A follower's frames keep coming; its next one starts the next clip.
      self.force_keyframe = true;
      self.timeline.stop_pts_ns(wall_ns)
    };
    if let Some(encoder) = &self.encoder {
      encoder.flush()?;
      let dropped = encoder.dropped();
      if dropped > 0 {
        eprintln!("Replay buffer encoder dropped {dropped} frames so far");
      }
    }

    let frames = self
      .frames
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (start_ns, video) = match from {
      ClipFrom::Latest {
        length_ns,
        since_ns,
      } => {
        let wanted = end_ns
          .saturating_sub(length_ns)
          .max(since_ns.unwrap_or(0))
          .max(0);
        // A clip of video can only start on a keyframe. The save before this
        // one left one exactly where it ended, so a clip that continues it
        // starts there; a clip of the full length starts at most a keyframe
        // interval early.
        let start = match &self.encoder {
          Some(_) => frames.keyframe_at_or_before(wanted).unwrap_or(wanted),
          None => wanted,
        };
        (start, frames.frames(start, end_ns))
      }
      ClipFrom::Follow { start_ns } => {
        let video = frames
          .keyframe_at_or_after(start_ns)
          .map(|first| frames.frames(first, end_ns))
          .unwrap_or_default();
        (start_ns, video)
      }
    };
    drop(frames);

    let video = match (&self.encoder, video.is_empty()) {
      (None, _) => None,
      (Some(_), true) => return Err("The replay buffer holds no video for this clip".to_owned()),
      (Some(_), false) => Some(VideoTake {
        frames: video,
        height: self.height,
        width: self.width,
      }),
    };
    Ok(ClipTake {
      start_ns,
      end_ns,
      video,
      system_audio: self
        .system_audio
        .as_ref()
        .map(|ring| ring.clip(start_ns, end_ns)),
      microphone: self
        .microphone
        .as_ref()
        .map(|(format, ring)| (*format, ring.clip(start_ns, end_ns))),
    })
  }

  /// Where the primary's clip ends: its last frame encoded again, as a
  /// keyframe, at the moment of the save. That frame holds a quiet screen to
  /// the true end, and is where the next clip starts.
  fn close_at(&mut self, wall_ns: i64) -> Result<i64, String> {
    if self.encoder.is_none() {
      return Ok(self.timeline.stop_pts_ns(wall_ns));
    }
    let Some(tail) = self.tail.take() else {
      return Ok(self.timeline.stop_pts_ns(wall_ns));
    };
    let pts = self.timeline.wall_frame_pts_ns(wall_ns);
    let encoded = self.encode(&tail, pts, true);
    self.tail = Some(tail);
    self.last_refresh_at = Some(Instant::now());
    encoded.ok_or_else(|| {
      self
        .failed
        .clone()
        .unwrap_or_else(|| "The replay buffer could not close the clip".to_owned())
    })
  }
}
