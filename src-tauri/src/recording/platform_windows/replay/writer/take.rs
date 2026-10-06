// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Answering a save: what this writer holds for the clip, on its own thread.

use super::super::encoder::Picture;
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
    // A frame that arrived before the save but was not yet due belongs in it.
    self.encode_staged();
    let wall_ns = self.elapsed_ns(at);
    let end_ns = if self.leads {
      self.close_at(wall_ns)?
    } else {
      // A follower's frames keep coming; its next one starts the next clip.
      self.force_keyframe = true;
      self.timeline.stop_pts_ns(wall_ns)
    };
    self.flush()?;

    let (start_ns, frames) = match from {
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
        let start = self.frames.keyframe_at_or_before(wanted).unwrap_or(wanted);
        (start, self.frames.frames(start, end_ns))
      }
      ClipFrom::Follow { start_ns } => {
        let frames = self
          .frames
          .keyframe_at_or_after(start_ns)
          .map(|first| self.frames.frames(first, end_ns))
          .unwrap_or_default();
        (start_ns, frames)
      }
    };
    if frames.is_empty() {
      return Err("The replay buffer holds no video for this clip".to_owned());
    }
    Ok(ClipTake {
      start_ns,
      end_ns,
      video: Some(VideoTake {
        frames,
        fps: self.config.fps.max(1),
        height: self.config.height,
        width: self.config.width,
      }),
    })
  }

  /// Where the primary's clip ends: its last frame encoded again, as a
  /// keyframe, at the moment of the save. That frame holds a quiet screen to
  /// the true end, and is where the next clip starts.
  fn close_at(&mut self, wall_ns: i64) -> Result<i64, String> {
    let Some(tail) = self.tail else {
      return Ok(self.timeline.stop_pts_ns(wall_ns));
    };
    let pts = self.timeline.wall_frame_pts_ns(wall_ns);
    let mut end = self.closing_frame(tail, pts)?;
    self.flush()?;
    // An encoder that turned out to ignore keyframe requests did not start
    // one here; encoding the frame again restarts it, which always does.
    if !self.is_keyframe(end) && self.encoder.ignores_forced_keyframes() {
      end = self.closing_frame(tail, end)?;
    }
    self.last_refresh_at = Some(Instant::now());
    Ok(end)
  }

  fn closing_frame(&mut self, tail: usize, pts: i64) -> Result<i64, String> {
    let end = self.encode(tail, pts, Picture::Keyframe);
    end.ok_or_else(|| {
      self
        .failed
        .clone()
        .unwrap_or_else(|| "The replay buffer could not close the clip".to_owned())
    })
  }

  fn is_keyframe(&self, pts_ns: i64) -> bool {
    self.frames.keyframe_at_or_after(pts_ns) == Some(pts_ns)
  }

  fn flush(&mut self) -> Result<(), String> {
    let flushed = self.encoder.flush(&mut self.encoded);
    self.keep_encoded();
    flushed.inspect_err(|error| self.fail(error.clone()))
  }
}
