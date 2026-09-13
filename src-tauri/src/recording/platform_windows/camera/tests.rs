// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn camera_becomes_ready_only_after_frames_and_time_have_warmed_up() {
  assert!(!warmup_complete(WARMUP_MIN_FRAMES - 1, WARMUP_DURATION));
  assert!(!warmup_complete(
    WARMUP_MIN_FRAMES,
    WARMUP_DURATION - Duration::from_millis(1)
  ));
  assert!(warmup_complete(WARMUP_MIN_FRAMES, WARMUP_DURATION));
}

#[test]
fn converts_rgba_to_native_bgra_and_mirrors_when_requested() {
  let rgba = [1, 2, 3, 4, 10, 20, 30, 40];
  assert_eq!(
    bgra_pixels(&rgba, 2, 1, false),
    [3, 2, 1, 4, 30, 20, 10, 40]
  );
  assert_eq!(bgra_pixels(&rgba, 2, 1, true), [30, 20, 10, 40, 3, 2, 1, 4]);
}

#[test]
fn native_capture_time_removes_webcam_delivery_latency() {
  let arrived = Instant::now();
  let epoch = Duration::from_secs(10_000);
  let (captured, source) = camera_frame_clock(
    Some(epoch - Duration::from_millis(120)),
    arrived,
    Some(epoch),
    arrived - Duration::from_secs(1),
  );
  assert_eq!(arrived.duration_since(captured), Duration::from_millis(120));
  assert_eq!(source, 99_998_800_000);
}

#[test]
fn implausible_camera_clock_uses_arrival_time() {
  let started = Instant::now();
  let arrived = started + Duration::from_millis(250);
  let (captured, source) = camera_frame_clock(
    Some(Duration::from_secs(1)),
    arrived,
    Some(Duration::from_secs(10)),
    started,
  );
  assert_eq!(captured, arrived);
  assert_eq!(source, 2_500_000);
}
