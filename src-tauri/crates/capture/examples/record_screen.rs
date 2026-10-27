// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Records the main display, and a camera if one is named, through the
//! app's own capture for a number of seconds, as Record does, and says how
//! each picture read: the movies and their `-picture.jsonl` land beside the
//! path given.
//!
//! `cargo run --release -p screenwide-capture --example record_screen -- <movie.mov> <seconds> [camera id]`
//!
//! The process that starts it needs Screen Recording permission, and
//! Camera permission for a camera.

#[cfg(target_os = "macos")]
fn main() {
  use std::sync::Arc;
  use std::time::{Duration, Instant};

  use screenwide_activity::{read_readings, states, PictureState};
  use screenwide_capture::capture::begin_blocking;
  use screenwide_capture::monitor::RecordingMonitor;
  use screenwide_capture::types::{CameraCaptureMode, CaptureStartupConfig, PrimaryCaptureSource};

  let mut args = std::env::args().skip(1);
  let path = std::path::PathBuf::from(args.next().expect("a movie path"));
  let seconds: f64 = args
    .next()
    .map_or(10.0, |value| value.parse().expect("seconds"));
  let camera = args.next().map(|device_id| CameraCaptureMode {
    device_id,
    flipped: false,
    fps: 30,
    height: 720,
    pal: false,
    width: 1280,
  });
  let camera_path = camera.as_ref().map(|_| path.with_file_name("camera.mov"));
  let started = begin_blocking(CaptureStartupConfig {
    camera,
    camera_path,
    include_own_windows: true,
    microphone_id: None,
    monitor: Arc::new(RecordingMonitor::default()),
    on_failure: Arc::new(|error| eprintln!("capture failed: {error}")),
    path: path.clone(),
    primary: PrimaryCaptureSource::Screen {
      fps: 60,
      monitor_id: core_graphics::display::CGDisplay::main().id,
      show_cursor: false,
    },
    system_audio: Default::default(),
  })
  .expect("the capture starts");
  started
    .first_frame
    .recv()
    .expect("a first frame")
    .expect("the first frame lands");
  eprintln!("recording {seconds} s to {}", path.display());
  std::thread::sleep(Duration::from_secs_f64(seconds));
  let finished = Instant::now();
  let info = started
    .session
    .stop_at(finished)
    .expect("the recording stops");
  let movies = std::iter::once(&info.path).chain(info.camera.as_ref().map(|camera| &camera.path));
  for movie in movies {
    let readings = read_readings(movie).expect("the picture readings");
    let found = states(&readings);
    let count = |state| found.iter().filter(|found| **found == state).count();
    println!(
      "{}: {} readings: still {}, moving {}, new {}",
      movie.display(),
      readings.len(),
      count(PictureState::Still),
      count(PictureState::Moving),
      count(PictureState::New),
    );
  }
}

#[cfg(not(target_os = "macos"))]
fn main() {
  eprintln!("This example records through macOS's capture.");
}
