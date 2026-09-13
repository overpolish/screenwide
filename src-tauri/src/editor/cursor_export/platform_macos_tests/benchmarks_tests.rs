// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
#[ignore = "set SCREENWIDE_GPU_BENCH_SOURCE to a 3600 x 2338 recording"]
fn benchmarks_retina_gpu_cursor_export() {
  let source = PathBuf::from(std::env::var("SCREENWIDE_GPU_BENCH_SOURCE").unwrap());
  let duration_ms = std::env::var("SCREENWIDE_GPU_BENCH_DURATION_MS")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(40_908);
  let resolution_scale_percent = std::env::var("SCREENWIDE_GPU_BENCH_SCALE_PERCENT")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(100);
  let directory = std::env::temp_dir().join(format!(
    "screenwide-gpu-export-benchmark-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let cursor_path = directory.join("source.cursor.jsonl");
  let destination = directory.join("output.mp4");
  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: 1_169.0,
        kind: CursorSourceKind::Screen,
        platform_id: "benchmark".to_owned(),
        video_height: 2_338,
        video_width: 3_600,
        width: 1_800.0,
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
      x: 300.0,
      y: 300.0,
    },
    CursorRecord::Position {
      timestamp_us: duration_ms * 1_000,
      x: 1_500.0,
      y: 800.0,
    },
  ];
  let json = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
  std::fs::write(&cursor_path, format!("{json}\n")).unwrap();
  let cancelled = AtomicBool::new(false);
  let started = std::time::Instant::now();
  let result = export(CursorExportRequest {
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
    duration_ms,
    height: 2_338,
    on_progress: &mut |_| {},
    output: &output(3_600, 2_338),
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VideoExportOptions {
      compression: 2,
      resolution_scale_percent,
      source_scale_percent: 100,
    },
    width: 3_600,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  eprintln!(
    "[cursor-export-benchmark] exported {:.2}s in {:.2}s to {}",
    duration_ms as f64 / 1_000.0,
    started.elapsed().as_secs_f64(),
    destination.display()
  );
}

#[test]
#[ignore = "set SCREENWIDE_GPU_BENCH_SOURCE to a recording"]
fn benchmarks_animated_mesh_export() {
  let source = PathBuf::from(std::env::var("SCREENWIDE_GPU_BENCH_SOURCE").unwrap());
  let duration_ms = std::env::var("SCREENWIDE_GPU_BENCH_DURATION_MS")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(30_000);
  let width = std::env::var("SCREENWIDE_GPU_BENCH_WIDTH")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(1_920);
  let height = std::env::var("SCREENWIDE_GPU_BENCH_HEIGHT")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(1_080);
  let directory = std::env::temp_dir().join(format!(
    "screenwide-mesh-export-benchmark-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let cancelled = AtomicBool::new(false);

  for (name, output) in [
    ("solid", {
      let mut settings = output(width, height);
      settings.crop_height = f64::from(height) * 0.84;
      settings.crop_width = f64::from(width) * 0.84;
      settings.crop_x = f64::from(width) * 0.08;
      settings.crop_y = f64::from(height) * 0.08;
      settings.image_width = f64::from(width) * 0.84;
      settings.image_x = f64::from(width) * 0.08;
      settings.image_y = f64::from(height) * 0.08;
      settings
    }),
    ("mesh", mesh_output(width, height)),
  ] {
    let destination = directory.join(format!("{name}.mp4"));
    let started = std::time::Instant::now();
    let result = export(CursorExportRequest {
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: None,
      camera_on_top: true,
      cancelled: &cancelled,
      cursor: None,
      cursor_effects: CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
      destination: &destination,
      duration_ms,
      height,
      on_progress: &mut |_| {},
      output: &output,
      screen: &source,
      selection: &TrackSelection::default(),
      timeline: None,
      video: VideoExportOptions {
        compression: 2,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
      width,
    })
    .unwrap();
    assert_eq!(result, ExportRunResult::Completed);
    eprintln!(
      "[mesh-export-benchmark] {name}: {:.2}s for {:.2}s, {} bytes",
      started.elapsed().as_secs_f64(),
      duration_ms as f64 / 1_000.0,
      std::fs::metadata(destination).unwrap().len(),
    );
  }
}

fn mesh_output(width: u32, height: u32) -> crate::screenshots::ScreenshotOutputSettings {
  use crate::screenshots::MeshGradientPoint;
  let mut output = output(width, height);
  output.background_type = "mesh".to_owned();
  output.mesh_colors = ["#112240", "#0ea5e9", "#8b5cf6", "#f97316", "#f8fafc"]
    .map(str::to_owned)
    .to_vec();
  output.mesh_points = vec![
    MeshGradientPoint {
      radius_x: 78.0,
      radius_y: 54.0,
      rotation: 18.0,
      x: 5.0,
      y: 12.0,
    },
    MeshGradientPoint {
      radius_x: 64.0,
      radius_y: 82.0,
      rotation: -24.0,
      x: 92.0,
      y: 10.0,
    },
    MeshGradientPoint {
      radius_x: 72.0,
      radius_y: 58.0,
      rotation: 42.0,
      x: 85.0,
      y: 92.0,
    },
    MeshGradientPoint {
      radius_x: 55.0,
      radius_y: 80.0,
      rotation: -10.0,
      x: 10.0,
      y: 88.0,
    },
  ];
  output.mesh_seed = 12_345;
  output.mesh_warp_percent = 8.0;
  output.crop_height = f64::from(height) * 0.84;
  output.crop_width = f64::from(width) * 0.84;
  output.crop_x = f64::from(width) * 0.08;
  output.crop_y = f64::from(height) * 0.08;
  output.image_width = f64::from(width) * 0.84;
  output.image_x = f64::from(width) * 0.08;
  output.image_y = f64::from(height) * 0.08;
  output
}
