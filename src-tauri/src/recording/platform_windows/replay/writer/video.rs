// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video onto the replay timeline and into the encoder.

use super::super::convert::SLOTS;
use super::super::encoder::Picture;
use super::*;

impl ReplayWriter {
  /// Takes one captured frame, reporting whether it was the first one
  /// encoded.
  pub(super) fn frame(&mut self, frame: Frame) -> bool {
    if self.failed.is_some() {
      return false;
    }
    let first = !self.timeline.has_started();
    if first && !self.start_timeline(&frame) {
      return false;
    }
    let wall_ns = self.elapsed_ns(frame.wall);
    let pts_ns = if self.config.wall_timestamped_frames {
      self.timeline.wall_frame_pts_ns(wall_ns)
    } else {
      self.timeline.frame_pts_ns(frame.source_ns(), wall_ns)
    };
    let Some(slot) = self.staging_slot() else {
      // Every texture is still being read: the encoder is behind, and this
      // frame is dropped as a full capture queue would drop it.
      return false;
    };
    if let Err(error) = self.converter.convert(&frame.texture, slot) {
      self.fail(format!("Direct3D could not convert a frame: {error}"));
      return false;
    }
    self.staged = Some(Staged { pts_ns, slot });
    self.last_frame_at = Some(Instant::now());
    if first || self.due(Instant::now()) {
      return self.encode_staged() && first;
    }
    false
  }

  /// Starts the timeline on this writer's first frame, the way the recording
  /// writer does, reporting whether the frame belongs on it.
  fn start_timeline(&mut self, frame: &Frame) -> bool {
    if self.config.establish_timeline_origin {
      let _ = self.config.timeline_origin.set(frame.wall);
    }
    // A camera beside the screen can be ready before the track that sets the
    // shared zero; its frames until then are not part of the replay.
    let Some(origin) = self.config.timeline_origin.get().copied() else {
      return false;
    };
    let offset_ns =
      i64::try_from(frame.wall.saturating_duration_since(origin).as_nanos()).unwrap_or(i64::MAX);
    self.timeline.start_at(
      frame.source_ns().saturating_sub(offset_ns),
      self.elapsed_ns(origin),
    );
    true
  }

  /// The texture the next frame is converted into: the staged frame's, which
  /// it replaces, or one neither the encoder nor a refresh still needs.
  fn staging_slot(&mut self) -> Option<usize> {
    if let Some(staged) = &self.staged {
      return Some(staged.slot);
    }
    let slot = match self.free_slot() {
      Some(slot) => slot,
      None => {
        self.collect();
        self.free_slot()?
      }
    };
    self.next_slot = (slot + 1) % SLOTS;
    Some(slot)
  }

  fn free_slot(&self) -> Option<usize> {
    (0..SLOTS)
      .map(|offset| (self.next_slot + offset) % SLOTS)
      .find(|&slot| Some(slot) != self.tail && !self.encoder.holds(slot))
  }

  pub(super) fn due(&self, now: Instant) -> bool {
    self
      .last_encoded_at
      .is_none_or(|at| now.saturating_duration_since(at) >= self.min_gap)
  }

  /// Encodes the staged frame, reporting whether it went to the encoder.
  pub(super) fn encode_staged(&mut self) -> bool {
    let Some(Staged { pts_ns, slot }) = self.staged.take() else {
      return false;
    };
    let picture = if self.follows_keyframe() {
      Picture::PreferKeyframe
    } else {
      Picture::Any
    };
    let encoded = self.encode(slot, pts_ns, picture).is_some();
    self.tail = Some(slot);
    self.last_encoded_at = Some(Instant::now());
    encoded
  }

  /// Whether a follower's next frame should be a keyframe: after a save, and
  /// after every keyframe the primary video wrote since the last one.
  fn follows_keyframe(&mut self) -> bool {
    if self.leads {
      return false;
    }
    let primary = self.keyframes.latest();
    let follow = std::mem::take(&mut self.force_keyframe) || primary > self.followed_keyframe_ns;
    self.followed_keyframe_ns = self.followed_keyframe_ns.max(primary);
    follow
  }

  /// Encodes the picture in `slot` at `pts_ns`, or a little later if that
  /// would crowd the frame before it, and reports where it went.
  pub(super) fn encode(&mut self, slot: usize, pts_ns: i64, picture: Picture) -> Option<i64> {
    if self.failed.is_some() {
      return None;
    }
    let pts_ns = self
      .last_encoded_ns
      .map_or(pts_ns, |last| pts_ns.max(last + MIN_FRAME_GAP_NS));
    // Media Foundation counts in 100ns units; the ring keeps the same instant.
    let pts_100ns = pts_ns / NANOS_PER_100NS;
    let result = self.encoder.encode(
      self.converter.texture(slot),
      slot,
      pts_100ns,
      self.frame_duration_100ns,
      picture,
      &mut self.encoded,
    );
    self.keep_encoded();
    match result {
      Ok(()) => {
        let pts_ns = pts_100ns * NANOS_PER_100NS;
        self.last_encoded_ns = Some(pts_ns);
        Some(pts_ns)
      }
      Err(error) => {
        self.fail(error);
        None
      }
    }
  }

  /// Moves what the encoder handed back into the ring.
  pub(super) fn keep_encoded(&mut self) {
    for frame in self.encoded.drain(..) {
      if self.leads && frame.keyframe {
        self.keyframes.publish(frame.pts_ns);
      }
      self.frames.push(frame);
    }
  }

  pub(super) fn collect(&mut self) {
    if let Err(error) = self.encoder.collect(&mut self.encoded) {
      self.fail(error);
    }
    self.keep_encoded();
  }

  /// Re-encodes the last frame of a screen that has stopped changing.
  fn refresh_keyframe(&mut self, now: Instant) {
    let Some(tail) = self.tail.filter(|_| self.leads && self.staged.is_none()) else {
      return;
    };
    if self.refresh_at().is_none_or(|at| now < at) {
      return;
    }
    let pts = self.timeline.wall_frame_pts_ns(self.elapsed_ns(now));
    self.encode(tail, pts, Picture::Keyframe);
    self.last_refresh_at = Some(now);
  }

  /// When a quiet screen's last frame is next due to be encoded again.
  fn refresh_at(&self) -> Option<Instant> {
    let since = match (self.last_frame_at, self.last_refresh_at) {
      (Some(frame), Some(refresh)) => frame.max(refresh),
      (frame, refresh) => frame.or(refresh)?,
    };
    Some(since + KEYFRAME_REFRESH)
  }

  /// Encodes whatever has come due, and collects what the encoder finished.
  pub(super) fn service(&mut self) {
    let now = Instant::now();
    if self.staged.is_some() && self.due(now) {
      self.encode_staged();
    }
    self.refresh_keyframe(now);
    if self.encoder.is_busy() {
      self.collect();
    }
  }

  /// How long the writer may wait for its next command before something
  /// else is due.
  pub(super) fn next_wake(&self) -> Duration {
    let now = Instant::now();
    let mut wake = KEYFRAME_REFRESH;
    if self.staged.is_some() {
      let due = self.last_encoded_at.map_or(now, |at| at + self.min_gap);
      wake = wake.min(due.saturating_duration_since(now));
    }
    if self.leads && self.tail.is_some() {
      if let Some(at) = self.refresh_at() {
        wake = wake.min(at.saturating_duration_since(now));
      }
    }
    if self.encoder.is_busy() {
      wake = wake.min(OUTPUT_POLL);
    }
    wake
  }
}
