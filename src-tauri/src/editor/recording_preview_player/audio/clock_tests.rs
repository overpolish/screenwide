// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use cpal::StreamInstant;

fn close(actual: f64, expected: f64) {
  assert!((actual - expected).abs() < 1e-7, "{actual} != {expected}");
}

#[test]
fn startup_waits_for_scheduled_playback_and_stalls_at_submitted_audio() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  close(clock.seconds_at(now + Duration::from_secs(1)), 0.0);
  clock.submit(
    OutputStreamTimestamp {
      callback: StreamInstant::from_nanos(1_000_000_000),
      playback: StreamInstant::from_nanos(1_130_000_000),
    },
    now,
    1024,
  );
  close(clock.seconds_at(now + Duration::from_millis(129)), 0.0);
  close(clock.seconds_at(now + Duration::from_millis(140)), 0.01);
  close(
    clock.seconds_at(now + Duration::from_secs(1)),
    1024.0 / 48_000.0,
  );
}

#[test]
fn future_callbacks_do_not_jump_the_current_playback_position() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  clock.schedule_buffer(now, now, 4800);
  clock.schedule_buffer(now + Duration::from_millis(100), now, 4800);
  close(clock.seconds_at(now + Duration::from_millis(50)), 0.05);
  close(clock.seconds_at(now + Duration::from_millis(110)), 0.11);
}

#[test]
fn display_refreshes_advance_smoothly_across_buffer_boundaries() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  for index in 0..100 {
    clock.schedule_buffer(
      now + Duration::from_secs_f64(index as f64 * 1024.0 / 48_000.0),
      now,
      1024,
    );
  }
  for frame in 0..120 {
    let elapsed = frame as f64 / 60.0;
    close(
      clock.seconds_at(now + Duration::from_secs_f64(elapsed)),
      elapsed,
    );
  }
}

#[test]
fn overdue_device_timestamp_maps_backwards_without_startup_jump() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  clock.submit(
    OutputStreamTimestamp {
      callback: StreamInstant::from_nanos(1_010_000_000),
      playback: StreamInstant::from_nanos(1_000_000_000),
    },
    now,
    1024,
  );
  close(clock.seconds_at(now), 0.01);
}

#[test]
fn pruning_keeps_the_sounding_buffer_until_the_next_one_starts() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  for index in 0..3 {
    clock.schedule_buffer(now + Duration::from_millis(index * 100), now, 4800);
  }
  clock.schedule_buffer(
    now + Duration::from_millis(300),
    now + Duration::from_millis(150),
    4800,
  );
  close(clock.seconds_at(now + Duration::from_millis(160)), 0.16);
  close(clock.seconds_at(now + Duration::from_millis(320)), 0.32);
}

#[test]
fn callback_arrival_jitter_does_not_change_the_device_timeline() {
  let clock = AudioClock::new(48_000);
  let now = Instant::now();
  for (index, jitter) in [0, 4, 1, 6].into_iter().enumerate() {
    let nanos = index as u64 * 100_000_000;
    clock.submit(
      OutputStreamTimestamp {
        callback: StreamInstant::from_nanos(nanos),
        playback: StreamInstant::from_nanos(nanos + 100_000_000),
      },
      now + Duration::from_millis(index as u64 * 100 + jitter),
      4800,
    );
  }
  close(clock.seconds_at(now + Duration::from_millis(450)), 0.35);
}
