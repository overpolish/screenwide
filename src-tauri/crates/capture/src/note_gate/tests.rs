// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;

/// The hold is process-wide, so these take turns.
static SERIAL: Mutex<()> = Mutex::new(());

struct Written(Mutex<Vec<(u64, f32)>>);

fn gate(written: &Arc<Written>) -> Arc<dyn Fn(Buffer) + Send + Sync> {
  let written = Arc::clone(written);
  gated(
    1,
    48_000,
    Arc::new(move |buffer: Buffer| {
      written
        .0
        .lock()
        .unwrap()
        .push((buffer.samples.len() as u64, buffer.samples[0]));
    }),
  )
}

fn buffer(captured_at: Instant, value: f32) -> Buffer {
  Buffer {
    captured_at,
    samples: vec![value; 4],
  }
}

fn values(written: &Arc<Written>) -> Vec<f32> {
  written
    .0
    .lock()
    .unwrap()
    .iter()
    .map(|(_, value)| *value)
    .collect()
}

#[test]
fn a_tap_leaves_the_recording_microphone_as_it_was() {
  let _turn = SERIAL
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let written = Arc::new(Written(Mutex::new(Vec::new())));
  let gate = gate(&written);
  let start = Instant::now();
  press(start, None);
  gate(buffer(start + Duration::from_millis(10), 1.0));
  gate(buffer(start + Duration::from_millis(20), 2.0));
  // Held back until the hold is known.
  assert!(values(&written).is_empty());
  release(start + Duration::from_millis(100));
  gate(buffer(start + Duration::from_millis(110), 3.0));
  assert_eq!(values(&written), vec![1.0, 2.0, 3.0]);
  cancel();
}

#[test]
fn a_note_is_silence_in_the_recording_from_key_down_to_release() {
  let _turn = SERIAL
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let written = Arc::new(Written(Mutex::new(Vec::new())));
  let gate = gate(&written);
  let start = Instant::now() - Duration::from_secs(1);
  gate(buffer(start - Duration::from_millis(10), 1.0));
  press(start, None);
  release(start + Duration::from_millis(500));
  gate(buffer(start + Duration::from_millis(10), 2.0));
  gate(buffer(start + Duration::from_millis(490), 3.0));
  gate(buffer(start + Duration::from_millis(510), 4.0));
  assert_eq!(values(&written), vec![1.0, 0.0, 0.0, 4.0]);
  // Every sample is still there, so the track keeps its length.
  assert!(written.0.lock().unwrap().iter().all(|(len, _)| *len == 4));
  cancel();
}

#[test]
fn a_note_that_could_not_be_recorded_lets_the_hold_through() {
  let _turn = SERIAL
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let written = Arc::new(Written(Mutex::new(Vec::new())));
  let gate = gate(&written);
  let start = Instant::now();
  press(start, None);
  gate(buffer(start + Duration::from_millis(10), 1.0));
  cancel();
  gate(buffer(start + Duration::from_millis(20), 2.0));
  assert_eq!(values(&written), vec![1.0, 2.0]);
}

#[test]
fn the_note_hears_only_the_hold() {
  let _turn = SERIAL
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let heard = Arc::new(Mutex::new(Vec::new()));
  let tee_heard = Arc::clone(&heard);
  let tee: NoteTee = Arc::new(move |samples, _, _, _| {
    tee_heard.lock().unwrap().push(samples[0]);
  });
  let written = Arc::new(Written(Mutex::new(Vec::new())));
  let gate = gate(&written);
  let start = Instant::now() - Duration::from_secs(1);
  gate(buffer(start - Duration::from_millis(10), 1.0));
  press(start, Some(tee));
  release(start + Duration::from_millis(400));
  gate(buffer(start + Duration::from_millis(100), 2.0));
  gate(buffer(start + Duration::from_millis(450), 3.0));
  assert_eq!(*heard.lock().unwrap(), vec![2.0]);
  cancel();
}
