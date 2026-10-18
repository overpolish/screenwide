// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;

#[test]
#[ignore = "uses the video path in SCREENWIDE_WINDOWS_PREVIEW_TEST"]
fn decodes_and_seeks_the_recorded_fragmented_mp4() {
  let path = std::env::var_os("SCREENWIDE_WINDOWS_PREVIEW_TEST")
    .map(std::path::PathBuf::from)
    .expect("set SCREENWIDE_WINDOWS_PREVIEW_TEST to a recording");
  let mut reader = NativeVideoReader::open(&path, 640, 360, 0).unwrap();
  let first = reader.frame_at(0).unwrap().unwrap();
  assert!(first.width <= 640 && first.height <= 360);
  assert!(first.rgba.iter().any(|value| *value != 0));
  reader.seek(5_000).unwrap();
  let later = reader.frame_at(5_000).unwrap().unwrap();
  assert_eq!(later.width, first.width);
  assert_eq!(later.height, first.height);
  let jpeg = encoded_jpeg(&later, 85).unwrap();
  assert!(jpeg.len() > 1_024);
  if let Some(output) = std::env::var_os("SCREENWIDE_WINDOWS_PREVIEW_FRAME") {
    std::fs::write(output, jpeg).unwrap();
  }
}

#[test]
#[ignore = "uses the video path in SCREENWIDE_WINDOWS_PREVIEW_TEST"]
fn decodes_video_from_an_sta_export_worker() {
  let path = std::env::var_os("SCREENWIDE_WINDOWS_PREVIEW_TEST")
    .map(std::path::PathBuf::from)
    .expect("set SCREENWIDE_WINDOWS_PREVIEW_TEST to a recording");
  std::thread::spawn(move || {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
      .ok()
      .unwrap();
    let decoded =
      NativeVideoReader::open(&path, 640, 360, 0).and_then(|mut reader| reader.frame_at(0));
    unsafe { CoUninitialize() };
    assert!(decoded.unwrap().is_some());
  })
  .join()
  .unwrap();
}
