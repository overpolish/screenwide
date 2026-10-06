// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn the_replay_buffer_shows_only_when_nothing_else_does() {
  assert_eq!(Shown::of(RecordingStatus::Idle, None, true), Shown::Replay);
  assert_eq!(
    Shown::of(RecordingStatus::Idle, None, false),
    Shown::Status(RecordingStatus::Idle)
  );
}

#[test]
fn a_countdown_shows_over_the_replay_buffer() {
  assert_eq!(
    Shown::of(RecordingStatus::Idle, Some(3), true),
    Shown::Countdown(3)
  );
}

#[test]
fn a_recording_shows_over_a_countdown_and_the_replay_buffer() {
  for status in [
    RecordingStatus::Starting,
    RecordingStatus::Recording,
    RecordingStatus::Paused,
    RecordingStatus::Stopping,
  ] {
    assert_eq!(Shown::of(status, Some(3), true), Shown::Status(status));
  }
}
