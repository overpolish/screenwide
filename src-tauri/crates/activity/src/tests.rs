// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::Cell;
use std::path::PathBuf;

use crate::grey::grey_from_bgra;
use crate::measure::{Measure, PictureReading, PictureState, SAMPLE_SIZE};
use crate::states::states;
use crate::tap::{read_readings, readings_path, PictureTap};

const CELLS: usize = SAMPLE_SIZE.0 * SAMPLE_SIZE.1;

fn flat(grey: u8) -> Vec<u8> {
  vec![grey; CELLS]
}

/// `base` with `rows` rows from `row` down set to `grey`.
fn with_block(base: u8, grey: u8, rows: usize, row: usize) -> Vec<u8> {
  let mut frame = flat(base);
  for cell in frame
    .iter_mut()
    .skip((row % SAMPLE_SIZE.1) * SAMPLE_SIZE.0)
    .take(rows * SAMPLE_SIZE.0)
  {
    *cell = grey;
  }
  frame
}

/// A frame unlike any other almost everywhere, as a first-person view
/// turning is: noise hashed from its index.
fn turning(index: usize) -> Vec<u8> {
  (0..CELLS)
    .map(|cell| {
      let mut x = (cell as u64) << 32 | (index as u64) << 1 | 1;
      x ^= x << 13;
      x ^= x >> 7;
      x ^= x << 17;
      (x % 251) as u8
    })
    .collect()
}

fn read(frames: impl IntoIterator<Item = Vec<u8>>) -> Vec<PictureReading> {
  let mut measure = Measure::new(10.0);
  frames
    .into_iter()
    .map(|frame| measure.next(frame))
    .collect()
}

#[test]
fn a_still_screen_stays_still_through_noise_below_the_change_level() {
  let readings = read((0..40).map(|index| flat(100 + (index % 3) as u8 * 4)));
  assert!(states(&readings)
    .iter()
    .all(|state| *state == PictureState::Still));
}

#[test]
fn scrolling_a_page_reads_as_moving_until_it_stops() {
  // A long page of rows unlike each other, still for ten seconds, scrolled
  // three rows a tenth of a second for four, then read for six.
  let page = |offset: usize| -> Vec<u8> {
    (0..CELLS)
      .map(|cell| ((cell / SAMPLE_SIZE.0 + offset) * 97 % 251) as u8)
      .collect()
  };
  let frames = (0..100)
    .map(|_| page(0))
    .chain((1..=40).map(|step| page(step * 3)))
    .chain((0..60).map(|_| page(120)));
  let found = states(&read(frames));
  assert!(found[100..135]
    .iter()
    .all(|state| *state == PictureState::Moving));
  assert_eq!(found.last(), Some(&PictureState::Still));
}

#[test]
fn on_a_still_screen_a_small_moving_part_is_moving_not_new() {
  // Ten seconds of a still screen, then a strip a tenth of the picture tall
  // moving down a row each frame, as a scrolling log does.
  let frames = (0..100)
    .map(|_| flat(100))
    .chain((0..20).map(|row| with_block(100, 220, 8, row)));
  let found = states(&read(frames));
  assert!(found[105..]
    .iter()
    .all(|state| *state == PictureState::Moving));
}

#[test]
fn on_a_still_screen_another_screen_is_new_then_settles() {
  let frames = (0..100).map(|_| flat(40)).chain((0..60).map(|_| flat(200)));
  let found = states(&read(frames));
  assert_eq!(found[100], PictureState::New);
  assert_eq!(found.last(), Some(&PictureState::Still));
}

#[test]
fn going_back_to_a_screen_seen_moments_ago_is_not_new() {
  // Two screens swapped every second, as flicking between two tabs.
  let frames = (0..200).map(|index| flat(if (index / 10) % 2 == 0 { 40 } else { 200 }));
  let found = states(&read(frames));
  assert!(found[30..].iter().all(|state| *state != PictureState::New));
}

#[test]
fn in_a_game_that_never_stops_moving_its_pause_screen_is_still() {
  // A minute of play, five seconds of a frozen death screen, more play. Its
  // first half second is new: the moment the view gives way to it.
  let frames = (0..600)
    .map(turning)
    .chain((0..50).map(|_| flat(30)))
    .chain((650..900).map(turning));
  let found = states(&read(frames));
  assert!(found[606..650]
    .iter()
    .all(|state| *state == PictureState::Still));
  let playing = found[..600]
    .iter()
    .filter(|state| **state != PictureState::Still)
    .count();
  assert!(
    playing > 590,
    "play read as moving or new: {playing} of 600"
  );
}

#[test]
fn on_a_screen_a_window_opening_is_new_even_after_a_video_has_played() {
  // Ten seconds still, ten of a video filling most of the screen, ten
  // still, then a window opening over the lower half.
  let video = |index: usize| {
    let mut frame = turning(index);
    frame[..CELLS / 10].fill(40);
    frame
  };
  let window = with_block(40, 200, SAMPLE_SIZE.1 / 2, SAMPLE_SIZE.1 / 2);
  let frames = (0..100)
    .map(|_| flat(40))
    .chain((100..200).map(video))
    .chain((0..100).map(|_| flat(40)))
    .chain((0..30).map(|_| window.clone()));
  let found = states(&read(frames));
  assert_eq!(found[300], PictureState::New);
  // Once it has played a while, a video's frames are not new screens.
  assert!(found[140..180]
    .iter()
    .all(|state| *state != PictureState::New));
}

#[test]
fn readings_are_timed_by_their_place_in_the_samples() {
  let times: Vec<_> = read((0..3).map(|_| flat(0)))
    .iter()
    .map(|reading| reading.ms)
    .collect();
  assert_eq!(times, [0, 100, 200]);
}

fn movie(name: &str) -> PathBuf {
  let directory =
    std::env::temp_dir().join(format!("screenwide-activity-{name}-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  directory.join("recording.mov")
}

#[test]
fn a_capture_at_sixty_frames_a_second_is_shrunk_once_per_sample() {
  let path = movie("sixty");
  let mut tap = PictureTap::create(&path).unwrap();
  let shrunk = Cell::new(0);
  for frame in 0..120_i64 {
    tap
      .frame(frame * 1_000_000_000 / 60, || {
        shrunk.set(shrunk.get() + 1);
        Some(flat(100))
      })
      .unwrap();
  }
  tap.finish(2_000_000_000).unwrap();
  assert_eq!(shrunk.get(), 20);
  assert_eq!(read_readings(&path).unwrap().len(), 20);
}

#[test]
fn a_screen_that_stops_sending_frames_reads_as_unchanged_until_it_changes() {
  // A frame at the start, nothing for three seconds as the screen sits
  // still, then another screen.
  let path = movie("idle");
  let mut tap = PictureTap::create(&path).unwrap();
  tap.frame(0, || Some(flat(40))).unwrap();
  tap.frame(3_000_000_000, || Some(flat(200))).unwrap();
  tap.finish(4_000_000_000).unwrap();
  let readings = read_readings(&path).unwrap();
  assert_eq!(readings.len(), 40);
  assert!(readings[1..30].iter().all(|reading| reading.changed == 0.0));
  assert_eq!(readings[30].ms, 3000);
  assert_eq!(readings[30].changed, 1.0);
  assert!(readings_path(&path).ends_with("recording-picture.jsonl"));
}

#[test]
fn a_frame_is_shrunk_to_cells_that_keep_where_it_is_light() {
  // A 400 by 300 BGRA frame, padded rows, black on its left half and white
  // on its right.
  let (width, height, stride) = (400, 300, 1664);
  let mut pixels = vec![0_u8; stride * height];
  for row in 0..height {
    for column in width / 2..width {
      pixels[row * stride + column * 4..][..4].copy_from_slice(&[255, 255, 255, 255]);
    }
  }
  let grey = grey_from_bgra(&pixels, width, height, stride);
  assert_eq!(grey.len(), CELLS);
  let (columns, _) = SAMPLE_SIZE;
  assert!(grey[..columns / 2].iter().all(|cell| *cell == 0));
  assert!(grey[columns / 2..columns].iter().all(|cell| *cell >= 250));
}
