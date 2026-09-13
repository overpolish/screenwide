// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::source_plan::{composed_region_plan, desktop_layout};
use super::*;
use crate::recording::{monitor::RecordingMonitor, CameraCaptureMode, SystemAudioSelection};

#[test]
#[ignore = "requires an interactive Windows camera and hardware encoder"]
fn records_a_playable_camera_sample() {
  let info = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
    .unwrap()
    .into_iter()
    .next()
    .expect("connect a camera before running this test");
  let format = crate::camera_format::available_camera_formats(info.index(), &[30])
    .unwrap()
    .into_iter()
    .next()
    .expect("the camera has no supported recording mode");
  let resolution = format.resolution();
  let path = std::env::temp_dir().join(format!(
    "screenwide-windows-camera-{}.mp4",
    std::process::id()
  ));
  let _ = std::fs::remove_file(&path);
  let start = begin_blocking(CaptureStartupConfig {
    camera: Some(CameraCaptureMode {
      device_id: crate::recording_inputs::camera_id(&info),
      flipped: false,
      fps: format.frame_rate(),
      height: resolution.height(),
      pal: false,
      width: resolution.width(),
    }),
    camera_path: None,
    include_own_windows: true,
    system_audio_skipped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    microphone_id: None,
    monitor: Arc::new(RecordingMonitor::default()),
    on_failure: Arc::new(|error| eprintln!("camera recording failure: {error}")),
    path: path.clone(),
    primary: PrimaryCaptureSource::Camera,
    system_audio: SystemAudioSelection::default(),
  })
  .unwrap();
  start
    .first_frame
    .recv_timeout(Duration::from_secs(10))
    .unwrap()
    .unwrap();
  std::thread::sleep(Duration::from_secs(2));
  let stopped_at = Instant::now();
  start.session.mark_stopped_at(stopped_at);
  let info = start.session.stop_at(stopped_at).unwrap();
  assert_eq!(
    info.primary_kind,
    super::super::encoding::PrimaryRecordingKind::Camera
  );
  assert_eq!(
    (info.width, info.height),
    (resolution.width(), resolution.height())
  );
  assert!(
    info.duration_ms >= 1_500,
    "duration was {} ms",
    info.duration_ms
  );
  assert!(std::fs::metadata(&path).unwrap().len() > 1_024);
  std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires an interactive Windows display, camera, and hardware encoders"]
fn records_synchronized_screen_and_camera_samples() {
  let monitor = xcap::Monitor::all().unwrap().into_iter().next().unwrap();
  let monitor_id = monitor.id().unwrap();
  let camera = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
    .unwrap()
    .into_iter()
    .next()
    .expect("connect a camera before running this test");
  let format = crate::camera_format::available_camera_formats(camera.index(), &[30])
    .unwrap()
    .into_iter()
    .next()
    .expect("the camera has no supported recording mode");
  let resolution = format.resolution();
  let directory = std::env::temp_dir();
  let screen_path = directory.join(format!(
    "screenwide-windows-screen-camera-{}.mp4",
    std::process::id()
  ));
  let camera_path = directory.join(format!(
    "screenwide-windows-camera-sidecar-{}.mp4",
    std::process::id()
  ));
  let _ = std::fs::remove_file(&screen_path);
  let _ = std::fs::remove_file(&camera_path);
  let start = begin_blocking(CaptureStartupConfig {
    camera: Some(CameraCaptureMode {
      device_id: crate::recording_inputs::camera_id(&camera),
      flipped: false,
      fps: format.frame_rate(),
      height: resolution.height(),
      pal: false,
      width: resolution.width(),
    }),
    camera_path: Some(camera_path.clone()),
    include_own_windows: true,
    system_audio_skipped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    microphone_id: None,
    monitor: Arc::new(RecordingMonitor::default()),
    on_failure: Arc::new(|error| eprintln!("screen/camera recording failure: {error}")),
    path: screen_path.clone(),
    primary: PrimaryCaptureSource::Screen {
      fps: 60,
      monitor_id,
      show_cursor: false,
    },
    system_audio: SystemAudioSelection::default(),
  })
  .unwrap();
  start
    .first_frame
    .recv_timeout(Duration::from_secs(10))
    .unwrap()
    .unwrap();
  std::thread::sleep(Duration::from_secs(2));
  let stopped_at = Instant::now();
  start.session.mark_stopped_at(stopped_at);
  let info = start.session.stop_at(stopped_at).unwrap();
  let camera_info = info.camera.expect("camera sidecar was not finalized");
  assert_eq!(
    (camera_info.width, camera_info.height),
    (resolution.width(), resolution.height())
  );
  assert!(
    info.duration_ms.abs_diff(camera_info.duration_ms) <= 100,
    "screen was {} ms but camera was {} ms",
    info.duration_ms,
    camera_info.duration_ms
  );
  assert!(std::fs::metadata(&screen_path).unwrap().len() > 1_024);
  assert!(std::fs::metadata(&camera_path).unwrap().len() > 1_024);
  std::fs::remove_file(screen_path).unwrap();
  std::fs::remove_file(camera_path).unwrap();
}

#[test]
#[ignore = "requires an interactive Windows display and hardware encoder"]
fn records_a_playable_screen_sample() {
  let monitor = xcap::Monitor::all().unwrap().into_iter().next().unwrap();
  let monitor_id = monitor.id().unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-windows-recording-{}.mp4",
    std::process::id()
  ));
  let _ = std::fs::remove_file(&path);
  let start = begin_blocking(CaptureStartupConfig {
    camera: None,
    camera_path: None,
    include_own_windows: true,
    system_audio_skipped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    microphone_id: None,
    monitor: Arc::new(RecordingMonitor::default()),
    on_failure: Arc::new(|error| eprintln!("recording failure: {error}")),
    path: path.clone(),
    primary: PrimaryCaptureSource::Screen {
      fps: 60,
      monitor_id,
      show_cursor: true,
    },
    system_audio: SystemAudioSelection::default(),
  })
  .unwrap();
  start
    .first_frame
    .recv_timeout(Duration::from_secs(5))
    .unwrap()
    .unwrap();
  std::thread::sleep(Duration::from_secs(1));
  let stopped_at = Instant::now();
  start.session.mark_stopped_at(stopped_at);
  // Reproduce a busy async finalizer: frames may keep arriving during this
  // delay, but none may extend the recording past the user's stop instant.
  std::thread::sleep(Duration::from_secs(3));
  let info = start.session.stop_at(stopped_at).unwrap();
  assert!(
    info.duration_ms >= 900,
    "duration was {} ms",
    info.duration_ms
  );
  assert!(
    info.duration_ms <= 1_500,
    "stop finalization added a frozen tail: {} ms",
    info.duration_ms
  );
  assert!(std::fs::metadata(&path).unwrap().len() > 1_024);
  std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires two Windows displays and a hardware encoder"]
fn records_a_playable_cross_monitor_region() {
  use tauri::{LogicalPosition, LogicalSize};

  let monitors = xcap::Monitor::all().unwrap();
  let displays = desktop_layout(&monitors).unwrap();
  assert!(displays.len() >= 2, "connect two displays for this test");
  let anchor = displays[0];
  let other = displays[1];
  let left = anchor.x.min(other.x);
  let top = anchor.y.min(other.y);
  let right = (anchor.x + anchor.width).max(other.x + other.width);
  let bottom = (anchor.y + anchor.height).max(other.y + other.height);
  let region = crate::recording::Region {
    position: LogicalPosition::new(left - anchor.x, top - anchor.y),
    size: LogicalSize::new(right - left, bottom - top),
  };
  let plan = composed_region_plan(&monitors, anchor.id, region)
    .unwrap()
    .expect("the test region must cross both displays");
  let path = std::env::temp_dir().join(format!(
    "screenwide-windows-cross-monitor-{}.mp4",
    std::process::id()
  ));
  let _ = std::fs::remove_file(&path);
  let start = begin_blocking(CaptureStartupConfig {
    camera: None,
    camera_path: None,
    include_own_windows: true,
    system_audio_skipped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    microphone_id: None,
    monitor: Arc::new(RecordingMonitor::default()),
    on_failure: Arc::new(|error| eprintln!("cross-monitor recording failure: {error}")),
    path: path.clone(),
    primary: PrimaryCaptureSource::Region {
      fps: 60,
      monitor_id: anchor.id,
      region,
      show_cursor: false,
    },
    system_audio: SystemAudioSelection::default(),
  })
  .unwrap();
  start
    .first_frame
    .recv_timeout(Duration::from_secs(10))
    .unwrap()
    .unwrap();
  std::thread::sleep(Duration::from_secs(1));
  let stopped_at = Instant::now();
  start.session.mark_stopped_at(stopped_at);
  let info = start.session.stop_at(stopped_at).unwrap();
  assert_eq!((info.width, info.height), (plan.width, plan.height));
  assert!(
    info.duration_ms >= 750,
    "duration was {} ms",
    info.duration_ms
  );
  assert!(std::fs::metadata(&path).unwrap().len() > 1_024);
  std::fs::remove_file(path).unwrap();
}
