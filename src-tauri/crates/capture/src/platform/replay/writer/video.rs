// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video onto the replay timeline and into the encoder.

use super::*;

impl ReplayWriter {
  /// Encodes one captured frame, reporting whether it went to the encoder.
  pub(super) fn frame(&mut self, frame: Frame) -> bool {
    let source_ns = match frame.clock {
      FrameClock::Source(source_ns) => source_ns,
      FrameClock::Wall => self.elapsed_ns(frame.wall),
    };
    if !self.timeline.has_started() && !self.primary_video {
      let Some(origin) = self.timeline_origin.get().copied() else {
        self.tail = Some(frame);
        return false;
      };
      let origin_ns = self.elapsed_ns(origin);
      let source_origin_ns = match frame.clock {
        FrameClock::Source(_) => {
          source_ns.saturating_sub(self.elapsed_ns(frame.wall).saturating_sub(origin_ns).max(0))
        }
        FrameClock::Wall => origin_ns,
      };
      self.timeline.start_at(source_origin_ns, origin_ns);
      self.origin_wall = Some(origin);
      // The last camera frame from before the screen's first is the honest
      // picture at time zero, as in a recording.
      if let Some(before) = self.tail.take() {
        let pts = self.timeline.frame_pts_ns(source_origin_ns, origin_ns);
        self.encode(&before, pts, true);
      }
    }
    let first = !self.timeline.has_started();
    if first {
      self.origin_source_ns = Some(source_ns);
      self.origin_wall = Some(frame.wall);
      let _ = self.timeline_origin.set(frame.wall);
    }
    let pts = self
      .timeline
      .frame_pts_ns(source_ns, self.elapsed_ns(frame.wall));
    let keyframe = first || self.follows_keyframe();
    let encoded = self.encode(&frame, pts, keyframe).is_some();
    self.tail = Some(frame);
    self.last_frame_at = Some(Instant::now());
    if first {
      self.flush_preroll();
    }
    encoded
  }

  /// Whether a follower's next frame should be a keyframe: after a save, and
  /// after every keyframe the primary video wrote since the last one.
  fn follows_keyframe(&mut self) -> bool {
    if self.primary_video {
      return false;
    }
    let primary = self.keyframes.0.load(Ordering::Acquire);
    let follow = std::mem::take(&mut self.force_keyframe) || primary > self.followed_keyframe_ns;
    self.followed_keyframe_ns = self.followed_keyframe_ns.max(primary);
    follow
  }

  /// Encodes `frame` at `pts_ns`, or a little later if that would crowd the
  /// frame before it, and reports where it went.
  pub(super) fn encode(&mut self, frame: &Frame, pts_ns: i64, keyframe: bool) -> Option<i64> {
    if self.failed.is_some() {
      return None;
    }
    let encoder = self.encoder.as_ref()?;
    let pts_ns = self
      .last_encoded_ns
      .map_or(pts_ns, |last| pts_ns.max(last + MIN_FRAME_GAP_NS));
    match encoder.encode(&frame.buf, pts_ns, keyframe) {
      Ok(()) => {
        self.stats.appended.fetch_add(1, Ordering::Relaxed);
        self.last_encoded_ns = Some(pts_ns);
        Some(pts_ns)
      }
      Err(error) => {
        self.fail(error);
        None
      }
    }
  }

  /// Re-encodes the last frame of a screen that has stopped changing.
  pub(super) fn refresh_keyframe(&mut self) {
    if !self.primary_video || !self.timeline.has_started() {
      return;
    }
    let now = Instant::now();
    let quiet =
      |since: Option<Instant>| since.is_none_or(|at| now.duration_since(at) >= KEYFRAME_REFRESH);
    if !quiet(self.last_frame_at) || !quiet(self.last_refresh_at) {
      return;
    }
    let Some(tail) = self.tail.take() else {
      return;
    };
    let pts = self.timeline.wall_frame_pts_ns(self.elapsed_ns(now));
    self.encode(&tail, pts, true);
    self.tail = Some(tail);
    self.last_refresh_at = Some(now);
  }
}
