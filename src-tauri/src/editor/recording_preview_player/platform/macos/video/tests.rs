// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn cached_native_sample_preserves_repeats_seek_and_end_of_stream() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-sample-cache-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let path = directory.join("frames.mp4");
  let status = std::process::Command::new(crate::editor::media_preview::ffmpeg_path())
    .args([
      "-y",
      "-hide_banner",
      "-loglevel",
      "error",
      "-f",
      "lavfi",
      "-i",
      "testsrc2=size=64x64:rate=10:duration=1",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&path)
    .status()
    .unwrap();
  assert!(status.success());
  let asset = open_asset(&path).unwrap();
  let mut reader = NativeVideoReader::open(&asset, 64, 64, 0, 1000).unwrap();
  let first = reader.frame_at(0).unwrap().unwrap();
  let repeated = reader.frame_at(1).unwrap().unwrap();
  assert_eq!(first.rgba, repeated.rgba);
  let later = reader.frame_at(700).unwrap().unwrap();
  assert_ne!(first.rgba, later.rgba);
  let end = reader.frame_at(2000).unwrap().unwrap();
  let repeated_end = reader.frame_at(2100).unwrap().unwrap();
  assert_eq!(end.rgba, repeated_end.rgba);
  reader.reset(0, 1000).unwrap();
  let reset = reader.frame_at(0).unwrap().unwrap();
  assert_eq!(first.rgba, reset.rgba);
  assert_eq!((reset.width, reset.height), (64, 64));
  let native = reader.pixel_frame_at(1).unwrap().unwrap();
  drop(reader);
  drop(asset);
  // Queued presentation owns its buffer independently of the decoder.
  // Check every channel against the existing CPU path after dropping it.
  unsafe extern "C" {
    fn CVPixelBufferGetIOSurface(pixels: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
  }
  assert!(
    !unsafe { CVPixelBufferGetIOSurface(native.as_ptr()) }.is_null(),
    "preview pixels must support direct Metal texture mapping"
  );
  let pixels = unsafe { &mut *native.as_ptr().cast::<cv::ImageBuf>() };
  assert_eq!((pixels.width(), pixels.height()), (64, 64));
  let flags = cv::pixel_buffer::LockFlags::READ_ONLY;
  unsafe { pixels.lock_base_addr(flags) }.result().unwrap();
  let base = unsafe { pixels.base_address() }.cast::<u8>();
  assert!(!base.is_null());
  let stride = pixels.bytes_per_row();
  let mut actual = Vec::with_capacity(reset.rgba.len());
  for row in 0..64 {
    let bytes = unsafe { std::slice::from_raw_parts(base.add(row * stride), 64 * 4) };
    for bgra in bytes.chunks_exact(4) {
      actual.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
    }
  }
  unsafe { pixels.unlock_lock_base_addr(flags) }
    .result()
    .unwrap();
  assert_eq!(actual, reset.rgba);
  drop(native);
  std::fs::remove_dir_all(directory).unwrap();
}
