// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "platform_macos_tests/benchmarks_tests.rs"]
mod benchmarks_tests;

use super::*;
use crate::{
  editor::CameraOverlaySettings,
  recording::cursor::{CursorRecord, CursorSource, CursorSourceKind, CursorStyle, FORMAT_VERSION},
};
use std::process::Command;

fn output(width: u32, height: u32) -> crate::screenshots::ScreenshotOutputSettings {
  let mut output = crate::screenshots::test_output_settings(width, height);
  output.background_color = "#000000".to_owned();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.mesh_seed = 0;
  output.mesh_warp_percent = 0.0;
  output.crop_height = f64::from(height);
  output.crop_width = f64::from(width);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(width);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output
}

#[test]
fn composites_keyboard_pixels_into_a_gpu_still() {
  use crate::editor::keyboard_effects::KeyboardOverlay;

  let source = crate::screenshots::CapturedImage {
    rgba: [0, 0, 0, 255].repeat(640 * 360),
    width: 640,
    height: 360,
  };
  let settings = output(640, 360);
  let plain = crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap();
  let mut keyboard = KeyboardOverlay {
    key_count: 1,
    animation: KeyboardOverlay::ANIMATION_POP,
    appearance: KeyboardOverlay::APPEARANCE_LIGHT,
    scale: 3.0,
    progress: 1.0,
    ..Default::default()
  };
  keyboard.keys[0].key_code = 55;
  keyboard.keys[0].visible = 1;
  keyboard.keys[0].progress = 1.0;
  keyboard.keys[0].alpha = 1.0;
  keyboard.keys[0].scale = 3.0;
  keyboard.keys[0].layout_progress = 1.0;
  keyboard.keys[0].layout_from_mask = 1;
  keyboard.keys[0].layout_to_mask = 1;
  let composed = crate::screenshots::compose_output_layers(
    &source,
    &settings,
    0.0,
    false,
    None,
    None,
    None,
    Some(&keyboard),
    false,
    false,
  )
  .unwrap();
  let changed = plain
    .rgba
    .chunks_exact(4)
    .zip(composed.rgba.chunks_exact(4))
    .filter(|(before, after)| before != after)
    .count();
  assert!(changed > 500, "the keyboard changed only {changed} pixels");
}

#[test]
fn exports_composited_cursor_pixels_into_a_real_movie() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-cursor-export-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let cursor_path = directory.join("source.cursor.jsonl");
  let destination = directory.join("output.mp4");
  let status = Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=320x180:r=30:d=1",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap();
  assert!(status.success());

  let mut records = vec![
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: 180.0,
        kind: CursorSourceKind::Screen,
        platform_id: "test".to_owned(),
        video_height: 180,
        video_width: 320,
        width: 320.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 24.0,
      hotspot_x: 1.0,
      hotspot_y: 1.0,
      style: CursorStyle::Arrow,
      timestamp_us: 0,
      width: 16.0,
    },
  ];
  records.extend((0..=50).map(|step| CursorRecord::Position {
    timestamp_us: step * 20_000,
    x: 80.0 + f64::from(step as u32) * 160.0 / 50.0,
    y: 80.0,
  }));
  let json = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
  std::fs::write(&cursor_path, format!("{json}\n")).unwrap();

  let cancelled = AtomicBool::new(false);
  let mut progress = Vec::new();
  let result = export(CursorExportRequest {
    annotation_track: crate::editor::annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: Some(&cursor_path),
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 1_000,
    height: 180,
    on_progress: &mut |position| progress.push(position),
    output: &output(320, 180),
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 50,
      source_scale_percent: 100,
    },
    width: 320,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  assert!(destination.is_file());
  assert!(progress.last().is_some_and(|position| *position > 900));
  let metadata = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-nostdin", "-i"])
    .arg(&destination)
    .output()
    .unwrap();
  assert!(
    String::from_utf8_lossy(&metadata.stderr).contains("Video: h264"),
    "the delivered cursor-baked recording must use compatible H.264 video"
  );

  // The export honours the resolution scale, so the delivered movie is
  // smaller than the canvas the cursor was recorded against. Read the
  // delivered size from the same helper the export sizes itself with, and
  // measure the cursor in those pixels rather than in the canvas's.
  let (exported_width, exported_height) = super::super::output_dimensions(
    320,
    180,
    VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 50,
      source_scale_percent: 100,
    },
  );
  let scale = f64::from(exported_width) / 320.0;
  let scaled = |value: u32| (f64::from(value) * scale).round() as u32;
  for (timestamp, expected_x) in [("0", scaled(80)), ("0.5", scaled(160))] {
    let frame = Command::new(media_preview::ffmpeg_path())
      .args(["-hide_banner", "-loglevel", "error", "-ss", timestamp, "-i"])
      .arg(&destination)
      .args([
        "-frames:v",
        "1",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "pipe:1",
      ])
      .output()
      .unwrap();
    assert!(frame.status.success());
    assert_eq!(
      frame.stdout.len(),
      exported_width as usize * exported_height as usize * 3
    );
    let lit = frame
      .stdout
      .chunks_exact(3)
      .enumerate()
      .filter(|(_, pixel)| pixel.iter().any(|channel| *channel > 200))
      .map(|(index, _)| (index % exported_width as usize, index / exported_width as usize))
      .collect::<Vec<_>>();
    assert!(
      !lit.is_empty(),
      "the frame at {timestamp}s should contain the cursor"
    );
    let left = lit.iter().map(|(x, _)| *x).min().unwrap();
    let right = lit.iter().map(|(x, _)| *x).max().unwrap();
    let top = lit.iter().map(|(_, y)| *y).min().unwrap();
    // Sample early while keeping the recorded hotspot at the drawn tip.
    let expected_x = expected_x as usize;
    let expected_y = scaled(80) as usize;
    // The tolerances travel with the picture: a half-size export draws a
    // half-size cursor, so the slack that framed it at full size would
    // otherwise let a badly placed one through.
    let slack = scaled(12) as usize;
    assert!(
      left.abs_diff(expected_x) <= slack,
      "the cursor at {timestamp}s starts at x={left}, expected about {expected_x}"
    );
    assert!(
      top.abs_diff(expected_y) <= slack,
      "the cursor at {timestamp}s starts at y={top}, expected about {expected_y}"
    );
    assert!(
      right - left <= scaled(48) as usize,
      "the cursor at {timestamp}s smeared from x={left} to x={right}"
    );
  }
  let timeline_destination = directory.join("timeline-output.mp4");
  let timeline = crate::editor::timeline_edit::TimelinePlan::from_edit(
    &crate::editor::timeline_edit::RecordingTimelineEdit {
      annotation_clips: Vec::new(),
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      next_segment_id: 2,
      segments: vec![
        crate::editor::timeline_edit::RecordingTimelineSegment {
          id: 0,
          source_end: 0.2,
          source_start: 0.0,
          playback_rate: 1.0,
        },
        crate::editor::timeline_edit::RecordingTimelineSegment {
          id: 1,
          source_end: 1.0,
          source_start: 0.8,
          playback_rate: 1.0,
        },
      ],
    },
    1_000,
  )
  .unwrap();
  let result = export(CursorExportRequest {
    annotation_track: crate::editor::annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: Some(&cursor_path),
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &timeline_destination,
    duration_ms: 1_000,
    height: 180,
    on_progress: &mut |_| {},
    output: &output(320, 180),
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: Some(&timeline),
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 320,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  assert!(media_preview::duration_ms(&timeline_destination)
    .is_some_and(|duration| duration.abs_diff(timeline.duration_ms()) < 100));
  let _ = std::fs::remove_dir_all(directory);
}

/// Fits the baked 28:40 arrow into arbitrary custom-cursor boxes without
/// stretching it. These tests pin both routing and hotspot-anchored geometry;
/// system artwork is unavailable outside a GUI session.
#[test]
fn exports_a_custom_cursor_at_the_fallback_arrows_aspect() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-custom-cursor-export-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let cursor_path = directory.join("source.cursor.jsonl");
  let destination = directory.join("output.mp4");
  let status = Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=320x180:r=30:d=1",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap();
  assert!(status.success());

  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: 180.0,
        kind: CursorSourceKind::Screen,
        platform_id: "test".to_owned(),
        video_height: 180,
        video_width: 320,
        width: 320.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 48.0,
      hotspot_x: 11.0,
      hotspot_y: 8.0,
      style: CursorStyle::Custom,
      timestamp_us: 0,
      width: 48.0,
    },
    CursorRecord::Position {
      timestamp_us: 0,
      x: 120.0,
      y: 50.0,
    },
    CursorRecord::Position {
      timestamp_us: 1_000_000,
      x: 120.0,
      y: 50.0,
    },
  ];
  let json = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
  std::fs::write(&cursor_path, format!("{json}\n")).unwrap();

  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: crate::editor::annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: Some(&cursor_path),
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 1_000,
    height: 180,
    on_progress: &mut |_| {},
    output: &output(320, 180),
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 320,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);

  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", "0.5", "-i"])
    .arg(&destination)
    .args([
      "-frames:v",
      "1",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "pipe:1",
    ])
    .output()
    .unwrap();
  assert!(frame.status.success());
  assert_eq!(frame.stdout.len(), 320 * 180 * 3);
  let lit = frame
    .stdout
    .chunks_exact(3)
    .enumerate()
    .filter(|(_, pixel)| pixel.iter().all(|channel| *channel > 200))
    .map(|(index, _)| (index % 320, index / 320))
    .collect::<Vec<_>>();
  assert!(!lit.is_empty(), "the custom cursor was not drawn");
  let left = lit.iter().map(|(x, _)| *x).min().unwrap();
  let right = lit.iter().map(|(x, _)| *x).max().unwrap();
  let top = lit.iter().map(|(_, y)| *y).min().unwrap();
  let bottom = lit.iter().map(|(_, y)| *y).max().unwrap();
  let width = (right - left + 1) as f64;
  let height = (bottom - top + 1) as f64;
  // The 28x40 arrow inside a 48x48 box draws about 24x39 of visible fill; the
  // stretched arrow that squashed the export drew about 34x39.
  assert!(
    height / width > 1.35,
    "the custom cursor drew {width}x{height}, which is not the fallback arrow's aspect"
  );
  // The recorded hotspot belongs to artwork that is not drawn, so the arrow's
  // own tip has to sit at the recorded position.
  assert!(
    left.abs_diff(120) <= 6 && top.abs_diff(50) <= 6,
    "the custom cursor's tip drew at ({left}, {top}) instead of (120, 50)"
  );
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn exports_camera_and_cursor_through_the_same_gpu_compositor() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-camera-cursor-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let camera = directory.join("camera.mov");
  let cursor_path = directory.join("source.cursor.jsonl");
  for (path, color, size) in [(&source, "black", "320x180"), (&camera, "red", "160x120")] {
    let status = Command::new(media_preview::ffmpeg_path())
      .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
      ])
      .arg(format!("color=c={color}:s={size}:r=30:d=1"))
      .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
      .arg(path)
      .status()
      .unwrap();
    assert!(status.success());
  }
  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: 180.0,
        kind: CursorSourceKind::Screen,
        platform_id: "test".to_owned(),
        video_height: 180,
        video_width: 320,
        width: 320.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 24.0,
      hotspot_x: 1.0,
      hotspot_y: 1.0,
      style: CursorStyle::Arrow,
      timestamp_us: 0,
      width: 16.0,
    },
    CursorRecord::Position {
      timestamp_us: 0,
      x: 80.0,
      y: 60.0,
    },
  ];
  let json = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
  std::fs::write(&cursor_path, format!("{json}\n")).unwrap();

  let cancelled = AtomicBool::new(false);
  for camera_on_top in [true, false] {
    let destination = directory.join(if camera_on_top {
      "camera-on-top.mp4"
    } else {
      "screen-on-top.mp4"
    });
    let result = export(CursorExportRequest {
      annotation_track: crate::editor::annotations::timing::AnnotationTrack::Primary,
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: Some((
        &camera,
        BakedVideoExportOptions {
          camera_drop_shadow: false,
          camera_height: 120,
          camera_width: 160,
          overlay: CameraOverlaySettings {
            camera_width: 80.0,
            camera_x: 100.0,
            camera_y: 70.0,
            frame_height: 60.0,
            frame_width: 80.0,
            frame_x: 60.0,
            frame_y: 40.0,
            radius_percent: 10.0,
          },
          screen_height: 180,
          screen_width: 320,
          video: VideoExportOptions {
            compression: 1,
            resolution_scale_percent: 100,
            source_scale_percent: 100,
          },
        },
      )),
      camera_on_top,
      cancelled: &cancelled,
      cursor: Some(&cursor_path),
      cursor_effects: CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
      destination: &destination,
      duration_ms: 1_000,
      height: 180,
      on_progress: &mut |_| {},
      output: &output(320, 180),
      screen: &source,
      selection: &TrackSelection::default(),
      timeline: None,
      video: VideoExportOptions {
        compression: 1,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
      width: 320,
    })
    .unwrap();
    assert_eq!(result, ExportRunResult::Completed);
    let frame = Command::new(media_preview::ffmpeg_path())
      .args(["-hide_banner", "-loglevel", "error", "-ss", "0.5", "-i"])
      .arg(&destination)
      .args([
        "-frames:v",
        "1",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "pipe:1",
      ])
      .output()
      .unwrap();
    assert!(frame.status.success());
    assert_eq!(
      frame
        .stdout
        .chunks_exact(3)
        .any(|pixel| pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 80),
      camera_on_top,
      "the camera should only remain visible when it is above the opaque screen"
    );
    let cursor_is_visible = frame
      .stdout
      .chunks_exact(3)
      .enumerate()
      .filter(|(index, _)| {
        let x = index % 320;
        let y = index / 320;
        (65..135).contains(&x) && (45..95).contains(&y)
      })
      .any(|(_, pixel)| pixel[0] > 180 && pixel[1] > 180 && pixel[2] > 180);
    assert_eq!(
      cursor_is_visible, !camera_on_top,
      "the cursor should follow the screen layer in the video stack"
    );
  }
  let _ = std::fs::remove_dir_all(directory);
}
