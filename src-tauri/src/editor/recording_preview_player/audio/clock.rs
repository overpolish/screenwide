// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Samples the device's scheduled output buffers at any display timestamp.
//! Never projects past submitted audio, including during startup or a stall.

#[cfg(any(target_os = "macos", test))]
use std::time::Duration;

use cpal::{OutputStreamTimestamp, StreamInstant};
use std::{collections::VecDeque, sync::Mutex, time::Instant};

#[derive(Clone, Copy)]
struct Buffer {
  playback_at: Instant,
  start_seconds: f64,
  duration_seconds: f64,
}

#[derive(Default)]
struct Schedule {
  origin: Option<(StreamInstant, Instant)>,
  submitted_frames: u64,
  buffers: VecDeque<Buffer>,
}

pub(crate) struct AudioClock {
  sample_rate: u32,
  schedule: Mutex<Schedule>,
}

impl AudioClock {
  pub(super) fn new(sample_rate: u32) -> Self {
    Self {
      sample_rate,
      schedule: Mutex::new(Schedule::default()),
    }
  }

  pub(super) fn submit(
    &self,
    timestamp: OutputStreamTimestamp,
    received_at: Instant,
    frames: usize,
  ) {
    // Map the device clock once. Re-anchoring every callback would turn
    // callback scheduling jitter into visible changes in playback velocity.
    let (device_origin, host_origin) = {
      let mut schedule = self
        .schedule
        .lock()
        .unwrap_or_else(|error| error.into_inner());
      *schedule
        .origin
        .get_or_insert((timestamp.callback, received_at))
    };
    let playback_at = if timestamp.playback >= device_origin {
      host_origin.checked_add(timestamp.playback.duration_since(device_origin))
    } else {
      host_origin.checked_sub(device_origin.duration_since(timestamp.playback))
    }
    .unwrap_or(received_at);
    self.schedule_buffer(playback_at, received_at, frames);
  }

  fn schedule_buffer(&self, playback_at: Instant, now: Instant, frames: usize) {
    let mut schedule = self
      .schedule
      .lock()
      .unwrap_or_else(|error| error.into_inner());
    // Keep the buffer currently sounding as well as all submitted future ones.
    while schedule.buffers.len() > 1 && schedule.buffers[1].playback_at <= now {
      schedule.buffers.pop_front();
    }
    let start_seconds = schedule.submitted_frames as f64 / f64::from(self.sample_rate);
    schedule.buffers.push_back(Buffer {
      playback_at,
      start_seconds,
      duration_seconds: frames as f64 / f64::from(self.sample_rate),
    });
    schedule.submitted_frames += frames as u64;
    // Bound memory even for a device returning a pathological future timestamp.
    while schedule.buffers.len() > 256 {
      schedule.buffers.pop_front();
    }
  }

  pub(crate) fn seconds_at(&self, time: Instant) -> f64 {
    let schedule = self
      .schedule
      .lock()
      .unwrap_or_else(|error| error.into_inner());
    let Some(buffer) = schedule
      .buffers
      .iter()
      .rev()
      .find(|buffer| buffer.playback_at <= time)
    else {
      return schedule
        .buffers
        .front()
        .map_or(0.0, |buffer| buffer.start_seconds);
    };
    buffer.start_seconds
      + time
        .saturating_duration_since(buffer.playback_at)
        .as_secs_f64()
        .min(buffer.duration_seconds)
  }

  pub(crate) fn seconds(&self) -> f64 {
    self.seconds_at(Instant::now())
  }

  #[cfg(target_os = "macos")]
  pub(crate) fn seconds_ahead(&self, seconds: f64) -> f64 {
    let ahead = if seconds.is_finite() {
      seconds.clamp(0.0, 0.05)
    } else {
      0.0
    };
    self.seconds_at(Instant::now() + Duration::from_secs_f64(ahead))
  }
}

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;
