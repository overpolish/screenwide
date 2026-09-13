// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Writer {
  pub(super) fn new(config: WriterConfig) -> Result<Self, String> {
    let sink = Sink::new(&config)?;
    Ok(Self {
      base: Instant::now(),
      frame_duration_100ns: 10_000_000_i64 / i64::from(config.fps.max(1)),
      config,
      failed: None,
      last_appended_ns: None,
      sink,
      tail: None,
      timeline: Timeline::default(),
    })
  }

  pub(super) fn elapsed_ns(&self, at: Instant) -> i64 {
    i64::try_from(at.saturating_duration_since(self.base).as_nanos()).unwrap_or(i64::MAX)
  }

  pub(super) fn append(&mut self, frame: &Frame, pts_ns: i64, duration_100ns: i64) -> bool {
    if self.failed.is_some() {
      return false;
    }
    match self
      .sink
      .write(frame, pts_ns / NANOS_PER_100NS, duration_100ns)
    {
      Ok(()) => {
        self.last_appended_ns = Some(pts_ns);
        true
      }
      Err(error) => {
        let reason = format!("Media Foundation stopped accepting video frames: {error}");
        (self.config.on_failure)(reason.clone());
        self.failed = Some(reason);
        false
      }
    }
  }

  pub(super) fn frame(&mut self, frame: Frame) -> bool {
    if after_stop(&self.config.stopped_at, frame.wall) {
      return false;
    }
    let frame = if let Some(crop) = self.config.source_crop {
      match crop_frame(&self.config.device, frame, crop) {
        Ok(frame) => frame,
        Err(error) => {
          let reason = format!("Direct3D could not crop the region frame: {error}");
          (self.config.on_failure)(reason.clone());
          self.failed = Some(reason);
          return false;
        }
      }
    } else if self.config.wall_timestamped_frames {
      match snapshot_frame(&self.config.device, frame) {
        Ok(frame) => frame,
        Err(error) => {
          let reason = format!("Direct3D could not cache the window frame: {error}");
          (self.config.on_failure)(reason.clone());
          self.failed = Some(reason);
          return false;
        }
      }
    } else {
      frame
    };
    if self.timeline.is_paused() {
      self.tail = Some(frame);
      return false;
    }
    let is_first = !self.timeline.has_started();
    let source_ns = frame.source_100ns.saturating_mul(NANOS_PER_100NS);
    if is_first {
      if self.config.establish_timeline_origin {
        let _ = self.config.timeline_origin.set(frame.wall);
      }
      let Some(origin) = self.config.timeline_origin.get().copied() else {
        // A secondary camera can become ready before the primary screen. Its
        // warm frames are deliberately discarded until the primary track
        // establishes the shared zero used by preview and export.
        self.tail = Some(frame);
        return false;
      };
      let offset_ns =
        i64::try_from(frame.wall.saturating_duration_since(origin).as_nanos()).unwrap_or(i64::MAX);
      self
        .timeline
        .start_at(source_ns.saturating_sub(offset_ns), self.elapsed_ns(origin));
    }
    if self.config.wall_timestamped_frames && !is_first {
      self.tail = Some(frame);
      return false;
    }
    let wall_ns = self.elapsed_ns(frame.wall);
    let pts_ns = if self.config.wall_timestamped_frames {
      self.timeline.wall_frame_pts_ns(wall_ns)
    } else {
      self.timeline.frame_pts_ns(source_ns, wall_ns)
    };
    let appended = self.append(&frame, pts_ns, self.frame_duration_100ns);
    self.tail = Some(frame);
    is_first && appended
  }

  pub(super) fn tick(&mut self, at: Instant) {
    if self.timeline.is_paused() || !self.timeline.has_started() {
      return;
    }
    let Some(frame) = self.tail.clone() else {
      return;
    };
    if after_stop(&self.config.stopped_at, at) {
      return;
    }
    let wall_ns = self.elapsed_ns(at);
    let pts_ns = self.timeline.wall_frame_pts_ns(wall_ns);
    self.append(&frame, pts_ns, self.frame_duration_100ns);
  }
  pub(super) fn finish(&mut self, at: Instant) -> Result<FinalizeInfo, String> {
    if !self.timeline.has_started() {
      return Err("The recording captured no frames".to_owned());
    }
    let stop_ns = self.timeline.stop_pts_ns(self.elapsed_ns(at));
    if let Some(tail) = self.tail.take() {
      self.append(&tail, stop_ns, 1);
      self.tail = Some(tail);
    }
    if let Some(error) = self.failed.take() {
      return Err(error);
    }
    self.sink.finish()?;
    let end_ns = self.last_appended_ns.unwrap_or_default();
    Ok(FinalizeInfo {
      camera: None,
      cursor_path: None,
      keyboard_path: None,
      duration_ms: u64::try_from(end_ns / 1_000_000).unwrap_or_default(),
      has_microphone: false,
      has_system_audio: false,
      height: self.config.height,
      path: self.config.path.clone(),
      primary_kind: self.config.primary_kind,
      source_scale_factor: 1.0,
      width: self.config.width,
    })
  }
}
