// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pin tracking benchmarks against real recordings. Run in release:
//!
//! ```text
//! SCREENWIDE_PIN_BENCH_SOURCE=recording.mov \
//! SCREENWIDE_PIN_BENCH_REGION=x0,y0,x1,y1 SCREENWIDE_PIN_BENCH_PINNED_MS=1500 \
//! cargo test --release --lib pin_bench -- --ignored --nocapture
//! ```
//!
//! Optional: `_START_MS`, `_END_MS`, `_SIDE` (tracking size, default `TRACKING_SIDE`),
//! `_CENTRE` (centre weighted), `_SHEET` (a PNG contact sheet of the tracked
//! box), `_TRUTH=scroll` for `truth5k.mov`'s known scroll.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::super::open_asset;
use super::super::NativeVideoReader;
use super::LumaReader;
use crate::editor::annotations::pin::model::PinKeyframe;
use crate::editor::annotations::pin::target::PinTarget;
use crate::editor::annotations::pin::{
  track_pin, FrameSource, LumaFrame, PinRequest, PinnedPath, TRACKING_SIDE,
};

fn var(name: &str) -> Option<String> {
  std::env::var(format!("SCREENWIDE_PIN_BENCH_{name}")).ok()
}

fn number(name: &str, fallback: u64) -> u64 {
  var(name)
    .and_then(|value| value.parse().ok())
    .unwrap_or(fallback)
}

fn duration_ms(path: &std::path::Path) -> u64 {
  let asset = open_asset(path).unwrap();
  (asset.duration().as_secs() * 1_000.0) as u64
}

/// Splits a source's reading time from the time spent in its callers.
struct Timed<'a> {
  inner: &'a mut LumaReader,
  decode: Duration,
  frames: usize,
}

impl FrameSource for Timed<'_> {
  fn source_size(&self) -> (u32, u32) {
    self.inner.source_size()
  }

  fn read(
    &mut self,
    start_ms: u64,
    end_ms: u64,
    each: &mut dyn FnMut(LumaFrame) -> bool,
  ) -> Result<(), String> {
    let started = Instant::now();
    let mut inside = Duration::ZERO;
    let mut frames = 0;
    let result = self.inner.read(start_ms, end_ms, &mut |frame| {
      frames += 1;
      let at = Instant::now();
      let keep = each(frame);
      inside += at.elapsed();
      keep
    });
    self.frames += frames;
    self.decode += started.elapsed() - inside;
    result
  }
}

struct Setup {
  path: PathBuf,
  duration_ms: u64,
  request: PinRequest,
  side: u32,
}

fn setup() -> Setup {
  let path = PathBuf::from(var("SOURCE").expect("set SCREENWIDE_PIN_BENCH_SOURCE"));
  let duration_ms = duration_ms(&path);
  let region: Vec<f64> = var("REGION")
    .expect("set SCREENWIDE_PIN_BENCH_REGION to x0,y0,x1,y1 in source pixels")
    .split(',')
    .map(|value| value.trim().parse().unwrap())
    .collect();
  let region = [region[0], region[1], region[2], region[3]];
  let request = PinRequest {
    start_ms: number("START_MS", 0),
    end_ms: number("END_MS", duration_ms).min(duration_ms),
    keyframes: vec![PinKeyframe {
      ms: number("PINNED_MS", 0),
      dx: 0.0,
      dy: 0.0,
    }],
    target: PinTarget {
      region,
      anchor: [0.5 * (region[0] + region[2]), 0.5 * (region[1] + region[3])],
      redaction: var("CENTRE").is_some(),
    },
  };
  Setup {
    path,
    duration_ms,
    request,
    side: number("SIDE", u64::from(TRACKING_SIDE)) as u32,
  }
}

/// `truth5k.mov`'s scroll: down 2000 pixels over three seconds, a second's
/// pause, and back over three, each an eased half cosine; the crop rounds
/// down to whole pixels.
fn truth_scroll(ms: u64) -> f64 {
  let t = ms as f64 / 1_000.0;
  let y = if t < 3.0 {
    1_000.0 * (1.0 - (std::f64::consts::PI * t / 3.0).cos())
  } else if t < 4.0 {
    2_000.0
  } else if t < 7.0 {
    2_000.0 - 1_000.0 * (1.0 - (std::f64::consts::PI * (t - 4.0) / 3.0).cos())
  } else {
    0.0
  };
  y.floor()
}

#[test]
#[ignore = "set SCREENWIDE_PIN_BENCH_SOURCE and _REGION to a recording"]
fn pin_bench_tracking() {
  let setup = setup();
  let request = &setup.request;
  println!(
    "source {} ({} ms), clip {}..{} ms, pinned {} ms, tracking side {}",
    setup.path.display(),
    setup.duration_ms,
    request.start_ms,
    request.end_ms,
    request.keyframes[0].ms,
    setup.side
  );

  // Decoding alone, forward through the clip.
  let mut reader = LumaReader::open(&setup.path, setup.duration_ms, setup.side).unwrap();
  let started = Instant::now();
  let mut frames = 0;
  let mut size = (0, 0);
  reader
    .read(request.start_ms, request.end_ms, &mut |frame| {
      frames += 1;
      size = (frame.width, frame.height);
      true
    })
    .unwrap();
  let decode_only = started.elapsed();
  println!(
    "decode only: {frames} frames at {}x{} in {:.0} ms, {:.0} fps",
    size.0,
    size.1,
    decode_only.as_secs_f64() * 1_000.0,
    frames as f64 / decode_only.as_secs_f64()
  );

  // The whole pin: chunked backward, forward, smoothed.
  let mut reader = LumaReader::open(&setup.path, setup.duration_ms, setup.side).unwrap();
  let mut timed = Timed {
    inner: &mut reader,
    decode: Duration::ZERO,
    frames: 0,
  };
  let started = Instant::now();
  let path = track_pin(&mut timed, request, &AtomicBool::new(false))
    .unwrap()
    .unwrap();
  let total = started.elapsed();
  let tracking = total - timed.decode;
  let clip_seconds = (request.end_ms - request.start_ms) as f64 / 1_000.0;
  println!(
    "pin: {} samples from {} decoded frames in {:.0} ms ({:.2}x the clip's length): decode {:.0} ms, tracking {:.0} ms ({:.2} ms a frame)",
    path.samples.len(),
    timed.frames,
    total.as_secs_f64() * 1_000.0,
    total.as_secs_f64() / clip_seconds,
    timed.decode.as_secs_f64() * 1_000.0,
    tracking.as_secs_f64() * 1_000.0,
    tracking.as_secs_f64() * 1_000.0 / path.samples.len().max(1) as f64
  );
  report(&path, request);
  if let Some(directory) = var("SHEET") {
    sheet(&setup, &path, &PathBuf::from(directory));
  }
}

fn report(path: &PinnedPath, request: &PinRequest) {
  let visible = path.samples.iter().filter(|sample| sample.visible).count();
  let clean = path
    .samples
    .iter()
    .filter(|sample| sample.confidence >= 0.99)
    .count();
  let lost = path
    .samples
    .iter()
    .filter(|sample| sample.visible && sample.confidence == 0.0)
    .count();
  println!(
    "samples: {} visible, {} matched against the pinned frame, {} bridged while on screen; flagged to check: {:?}",
    visible, clean, lost, path.weak
  );
  if var("TRUTH").as_deref() == Some("scroll") {
    let pinned = truth_scroll(request.keyframes[0].ms);
    let mut errors: Vec<f64> = path
      .samples
      .iter()
      .filter(|sample| sample.visible)
      .map(|sample| {
        let expected = pinned - truth_scroll(sample.ms);
        (f64::from(sample.dy) - expected)
          .abs()
          .max(f64::from(sample.dx).abs())
      })
      .collect();
    errors.sort_by(f64::total_cmp);
    let pick = |share: f64| errors[((errors.len() - 1) as f64 * share) as usize];
    println!(
      "error against the known scroll, in source pixels: median {:.2}, p95 {:.2}, max {:.2}",
      pick(0.5),
      pick(0.95),
      pick(1.0)
    );
    let wrongly_shown = path
      .samples
      .iter()
      .filter(|sample| {
        let y = request.target.anchor[1] + pinned - truth_scroll(sample.ms);
        sample.visible && !(0.0..2_338.0).contains(&y)
      })
      .count();
    println!("frames shown while the anchor is off the frame: {wrongly_shown}");
  }
}

/// Every so many frames at the tracking size, with the followed box drawn:
/// green matched, amber bridged or weak, and no box while hidden.
fn sheet(setup: &Setup, path: &PinnedPath, directory: &std::path::Path) {
  const TILES: usize = 24;
  let request = &setup.request;
  let mut reader = LumaReader::open(&setup.path, setup.duration_ms, 640).unwrap();
  let (source_width, _) = reader.source_size();
  let every = (path.samples.len() / TILES).max(1);
  let wanted: Vec<u64> = path
    .samples
    .iter()
    .step_by(every)
    .map(|sample| sample.ms)
    .collect();
  let mut tiles = Vec::new();
  reader
    .read(request.start_ms, request.end_ms, &mut |frame| {
      if wanted.contains(&frame.ms) {
        tiles.push(frame);
      }
      true
    })
    .unwrap();
  let (width, height) = (tiles[0].width as usize, tiles[0].height as usize);
  let columns = 6;
  let rows = tiles.len().div_ceil(columns);
  let mut image = image::RgbImage::new((width * columns) as u32, (height * rows) as u32);
  for (index, tile) in tiles.iter().enumerate() {
    let (ox, oy) = ((index % columns) * width, (index / columns) * height);
    for y in 0..height {
      for x in 0..width {
        let value = tile.pixels[y * width + x];
        image.put_pixel((ox + x) as u32, (oy + y) as u32, image::Rgb([value; 3]));
      }
    }
    let Some(sample) = path.samples.iter().find(|sample| sample.ms == tile.ms) else {
      continue;
    };
    if !sample.visible {
      continue;
    }
    let factor = width as f64 / f64::from(source_width);
    let [ax, ay] = request.target.anchor;
    let scale = f64::from(sample.scale);
    let corner = |x: f64, y: f64| {
      (
        ((ax + (x - ax) * scale + f64::from(sample.dx)) * factor) as i64,
        ((ay + (y - ay) * scale + f64::from(sample.dy)) * factor) as i64,
      )
    };
    let (x0, y0) = corner(request.target.region[0], request.target.region[1]);
    let (x1, y1) = corner(request.target.region[2], request.target.region[3]);
    let colour = if sample.confidence >= 0.7 {
      image::Rgb([40, 220, 60])
    } else {
      image::Rgb([255, 170, 0])
    };
    for thickness in 0..2 {
      for x in x0..=x1 {
        for y in [y0 + thickness, y1 - thickness] {
          if (0..width as i64).contains(&x) && (0..height as i64).contains(&y) {
            image.put_pixel((ox as i64 + x) as u32, (oy as i64 + y) as u32, colour);
          }
        }
      }
      for y in y0..=y1 {
        for x in [x0 + thickness, x1 - thickness] {
          if (0..width as i64).contains(&x) && (0..height as i64).contains(&y) {
            image.put_pixel((ox as i64 + x) as u32, (oy as i64 + y) as u32, colour);
          }
        }
      }
    }
  }
  std::fs::create_dir_all(directory).unwrap();
  let name = setup
    .path
    .file_stem()
    .map(|stem| stem.to_string_lossy().into_owned())
    .unwrap_or_default();
  let output = directory.join(format!(
    "{name}-{}-{}x{}.png",
    request.keyframes[0].ms, request.target.region[0], request.target.region[1]
  ));
  image.save(&output).unwrap();
  println!("sheet: {}", output.display());
}

/// Plays the recording at 60 fps the way the preview decodes it, and counts
/// the frames that arrive late: alone, then while a pin is tracked.
#[test]
#[ignore = "set SCREENWIDE_PIN_BENCH_SOURCE and _REGION to a recording"]
fn pin_bench_playback_while_tracking() {
  let setup = setup();
  let play = |label: &str, tracking: Option<Arc<AtomicBool>>| {
    let asset = open_asset(&setup.path).unwrap();
    let (width, height) = {
      let reader = LumaReader::open(&setup.path, setup.duration_ms, u32::MAX).unwrap();
      reader.source_size()
    };
    // The preview decodes a Retina pane at about half its source size.
    let (pane_width, pane_height) = ((width / 2) & !1, (height / 2) & !1);
    let seconds = number("PLAY_MS", 5_000).min(setup.duration_ms);
    let mut reader =
      NativeVideoReader::open(&asset, pane_width, pane_height, 0, setup.duration_ms).unwrap();
    let frame = Duration::from_micros(16_667);
    let started = Instant::now();
    let (mut late, mut worst, mut count) = (0, Duration::ZERO, 0);
    let mut index = 0_u64;
    while index * 16_667 / 1_000 < seconds {
      let deadline = frame * index as u32;
      let before = Instant::now();
      reader.pixel_frame_at(index * 16_667 / 1_000).unwrap();
      worst = worst.max(before.elapsed());
      if started.elapsed() > deadline + frame {
        late += 1;
      }
      count += 1;
      index += 1;
      if let Some(wait) = (deadline + frame).checked_sub(started.elapsed()) {
        std::thread::sleep(wait);
      }
    }
    if let Some(stop) = tracking {
      stop.store(true, Ordering::Relaxed);
    }
    println!(
      "{label}: {count} frames at {pane_width}x{pane_height}, {late} late, slowest decode {:.1} ms",
      worst.as_secs_f64() * 1_000.0
    );
  };
  play("playback alone", None);

  let stop = Arc::new(AtomicBool::new(false));
  let tracker_stop = Arc::clone(&stop);
  let path = setup.path.clone();
  let request = setup.request.clone();
  let (duration_ms, side) = (setup.duration_ms, setup.side);
  let tracker = std::thread::spawn(move || {
    let mut frames = 0;
    let started = Instant::now();
    while !tracker_stop.load(Ordering::Relaxed) {
      let mut reader = LumaReader::open(&path, duration_ms, side).unwrap();
      if let Some(result) = track_pin(&mut reader, &request, &tracker_stop).unwrap() {
        frames += result.samples.len();
      }
    }
    (frames, started.elapsed())
  });
  play("playback while tracking", Some(stop));
  let (frames, elapsed) = tracker.join().unwrap();
  println!(
    "tracking alongside: {frames} frames tracked, {:.0} frames a second",
    frames as f64 / elapsed.as_secs_f64()
  );
}
