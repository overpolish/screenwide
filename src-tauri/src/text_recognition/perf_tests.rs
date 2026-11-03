// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Times reading a busy page, stage by stage, as the app reads an area drawn
//! over all of it, and checks reading text and QR codes at once finds what
//! each finds alone. Run by hand on a screenshot:
//!
//! SCREENWIDE_OCR_PAGE=page.png cargo test --lib perf_tests -- --ignored --nocapture
//!
//! A dev build is timed, as `pnpm tauri dev` runs one.

use std::time::Instant;

use screenwide_imaging::CapturedImage;
use screenwide_osc::desktop::DesktopDisplay;
use screenwide_osc::geometry::{Point, Rect, Size};

use super::*;

fn page(width: u32, height: u32) -> CapturedImage {
  let path = std::env::var("SCREENWIDE_OCR_PAGE").expect("SCREENWIDE_OCR_PAGE names a picture");
  let image = image::open(path).expect("the page opens").into_rgba8();
  let image = image::imageops::resize(&image, width, height, image::imageops::FilterType::Triangle);
  CapturedImage {
    width,
    height,
    rgba: image.into_raw(),
  }
}

fn timed<T>(
  rows: &mut Vec<(&'static str, f64)>,
  stage: &'static str,
  run: impl FnOnce() -> T,
) -> T {
  let start = Instant::now();
  let value = run();
  rows.push((stage, start.elapsed().as_secs_f64() * 1000.0));
  value
}

fn texts(lines: &[RecognizedLine]) -> Vec<&str> {
  lines.iter().map(|line| line.text.as_str()).collect()
}

fn run(width: u32, height: u32, scale: f64) {
  let picture = page(width, height);
  let size = Size {
    width: f64::from(width) / scale,
    height: f64::from(height) / scale,
  };
  let display = DesktopDisplay {
    id: 1,
    origin: Point::default(),
    size,
    scale,
  };
  // Vision's models load on the first read; the app warms them at launch.
  macos::platform::recognize(&picture.rgba, width, height).unwrap();
  let state = TextRecognitionState::default();
  let generation = state.begin();
  state.install(generation, [(1, scale, picture)]);
  let mut rows = Vec::new();

  timed(&mut rows, "Crop and join the area", || {
    let area = Rect::from_xywh(0.0, 0.0, size.width, size.height);
    state.select_desktop_region(&[display], 1, area).unwrap();
  });
  let (generation, image) = timed(&mut rows, "Hand the area to the recognizer", || {
    state.recognition_input().unwrap()
  });
  let qr_codes = timed(&mut rows, "QR codes alone", || {
    screenwide_qr::read(&image.rgba, image.width, image.height)
  });
  let lines = timed(
    &mut rows,
    "Text alone (Vision, with conversion and boxes)",
    || macos::platform::recognize(&image.rgba, image.width, image.height).unwrap(),
  );
  let (both_lines, both_codes) = timed(&mut rows, "Text and QR codes, as the app reads", || {
    tauri::async_runtime::block_on(recognition_worker::recognize(image)).unwrap()
  });
  assert_eq!(
    texts(&both_lines),
    texts(&lines),
    "the text read with the QR codes"
  );
  assert_eq!(
    both_codes
      .iter()
      .map(|code| (code.content.as_str(), code.decode_error.is_some()))
      .collect::<Vec<_>>(),
    qr_codes
      .iter()
      .map(|code| (
        code.content.as_deref().unwrap_or_default(),
        code.content.is_none()
      ))
      .collect::<Vec<_>>(),
    "the QR codes read with the text",
  );
  let marks = timed(
    &mut rows,
    "Install, the marks' packet, the island's state",
    || {
      let text = texts(&both_lines).join("\n");
      let result = TextRecognitionResult {
        lines: both_lines,
        qr_codes: both_codes,
        text,
      };
      assert!(state.install_result(generation, result));
      let snapshot = state.visual_snapshot(generation).unwrap();
      assert!(state.island_state().is_some());
      visual::RenderPacket::ready(&snapshot).rects.len()
    },
  );

  eprintln!(
    "\n{width}x{height} at {scale}x: {} lines; {} QR codes, {} unread; {marks} marks",
    lines.len(),
    qr_codes.len(),
    qr_codes
      .iter()
      .filter(|code| code.content.is_none())
      .count(),
  );
  eprintln!("| Stage | ms |\n| --- | ---: |");
  for (stage, time) in rows {
    eprintln!("| {stage} | {time:.1} |");
  }
}

#[test]
#[ignore = "times reading the page named by SCREENWIDE_OCR_PAGE"]
fn reads_a_busy_page_in_one_wait() {
  run(1568, 931, 1.0);
  run(3136, 1862, 2.0);
}
