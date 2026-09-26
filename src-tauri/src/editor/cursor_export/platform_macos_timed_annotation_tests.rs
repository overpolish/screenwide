// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::arrow::new_arrow;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::AnnotationPoint;
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::process::Command;

/// How many of the arrow's own yellow pixels one frame of the exported movie
/// holds, which is the only measure of the reveal that survives the encode.
fn exported_yellow(destination: &std::path::Path, time: &str, width: u32, height: u32) -> usize {
  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", time, "-i"])
    .arg(destination)
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
  assert_eq!(frame.stdout.len(), width as usize * height as usize * 3);
  frame
    .stdout
    .chunks_exact(3)
    .filter(|rgb| rgb[0] > 160 && rgb[1] > 100 && rgb[2] < 80)
    .count()
}

#[test]
fn exports_timed_arrows_across_a_cut_and_speed_change() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-timed-arrow-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let camera = directory.join("camera.mov");
  let destination = directory.join("output.mp4");
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=320x180:r=2:d=3",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p"
    ])
    .arg(&source)
    .status()
    .unwrap()
    .success());
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=red:s=160x120:r=2:d=3",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p"
    ])
    .arg(&camera)
    .status()
    .unwrap()
    .success());
  let mut arrow = new_arrow(
    "timed-arrow".into(),
    AnnotationPoint { x: 60.0, y: 90.0 },
    AnnotationPoint { x: 260.0, y: 90.0 },
    None,
  );
  arrow.style.width = 16.0;
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 2,
      keyboard_deletions: Box::default(),
      annotation_clips: vec![RecordingAnnotationClip {
        pin: None,
        annotation: arrow,
        track_id: AnnotationTrack::Primary,
        start_ms: 550,
        end_ms: 2500,
      }],
      segments: vec![
        RecordingTimelineSegment {
          id: 0,
          source_start: 0.0,
          source_end: 1.0 / 3.0,
          playback_rate: 1.0,
        },
        RecordingTimelineSegment {
          id: 1,
          source_start: 0.5,
          source_end: 1.0,
          playback_rate: 2.0,
        },
      ],
    },
    3000,
  )
  .unwrap();
  let mut output = crate::screenshots::test_output_settings(320, 180);
  output.background_color = "#000000".into();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.crop_width = 320.0;
  output.crop_height = 180.0;
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = 320.0;
  output.image_x = 0.0;
  output.image_y = 0.0;
  for baked in [false, true] {
    for scale in [100, 75, 50] {
      let (width, height) = super::super::output_dimensions(
        320,
        180,
        VideoExportOptions {
          compression: 0,
          resolution_scale_percent: scale,
          source_scale_percent: 100,
        },
      );
      let cancelled = AtomicBool::new(false);
      assert_eq!(
        export(CursorExportRequest {
          annotation_track: AnnotationTrack::Primary,
          audio_layout: AudioLayout::SeparateTracks,
          audio_source: None,
          camera: baked.then_some((
            &camera,
            BakedVideoExportOptions {
              camera_drop_shadow: false,
              camera_width: 160,
              camera_height: 120,
              screen_width: 320,
              screen_height: 180,
              overlay: crate::editor::CameraOverlaySettings {
                camera_width: 40.0,
                camera_x: 20.0,
                camera_y: 15.0,
                frame_width: 40.0,
                frame_height: 30.0,
                frame_x: 0.0,
                frame_y: 0.0,
                radius_percent: 0.0,
              },
              video: VideoExportOptions {
                compression: 0,
                resolution_scale_percent: 100,
                source_scale_percent: 100
              },
            }
          )),
          camera_on_top: true,
          cancelled: &cancelled,
          cursor: None,
          cursor_effects: CursorEffectSettings::default(),
          keyboard: None,
          keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
          destination: &destination,
          duration_ms: 3000,
          height: 180,
          on_progress: &mut |_| {},
          output: &output,
          screen: &source,
          selection: &TrackSelection::default(),
          timeline: Some(&timeline),
          video: VideoExportOptions {
            compression: 0,
            resolution_scale_percent: scale,
            source_scale_percent: 100
          },
          width: 320,
        })
        .unwrap(),
        ExportRunResult::Completed
      );
      // Sparse screen samples must still produce a regular effect animation
      // clock.
      let frames = Command::new(media_preview::ffmpeg_path())
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(&destination)
        .args([
          "-fps_mode",
          "passthrough",
          "-f",
          "rawvideo",
          "-pix_fmt",
          "rgb24",
          "pipe:1",
        ])
        .output()
        .unwrap();
      assert!(frames.status.success());
      assert_eq!(
        frames.stdout.len(),
        105 * width as usize * height as usize * 3
      );
      // Source 0.95 and 1.9 seconds are inside the same annotation across the
      // cut, far enough into its draw-in that the annotation is most of its
      // full size.
      for (time, visible) in [
        ("0.2", false),
        ("0.95", true),
        ("1.2", true),
        ("1.65", false),
      ] {
        let frame = Command::new(media_preview::ffmpeg_path())
          .args(["-hide_banner", "-loglevel", "error", "-ss", time, "-i"])
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
        assert_eq!(frame.stdout.len(), width as usize * height as usize * 3);
        let red = frame
          .stdout
          .chunks_exact(3)
          .filter(|rgb| rgb[0] > 160 && rgb[1] < 80 && rgb[2] < 80)
          .count();
        assert_eq!(red > 50, baked, "camera visibility at {scale}%");
        let yellow = frame
          .stdout
          .chunks_exact(3)
          .filter(|rgb| rgb[0] > 160 && rgb[1] > 100 && rgb[2] < 80)
          .count();
        if visible {
          let yellow_x: Vec<_> = frame
            .stdout
            .chunks_exact(3)
            .enumerate()
            .filter(|(_, rgb)| rgb[0] > 160 && rgb[1] > 100 && rgb[2] < 80)
            .map(|(index, _)| index % width as usize)
            .collect();
          assert!(
            yellow_x
              .iter()
              .all(|x| *x > width as usize / 10 && *x < width as usize * 9 / 10),
            "scaled arrow must retain its placement at {scale}%"
          );
        }
        assert!(
          if visible {
            yellow > (1000 * u32::from(scale).pow(2) / 10000) as usize
          } else {
            yellow < 10
          },
          "at {time}s, found {yellow} yellow pixels; expected visible={visible}"
        );
      }
      if scale == 100 && !baked {
        // The clip runs from source 550ms to 2500ms, so each phase is a third
        // of it rather than the whole three quarters of a second: the arrow
        // draws itself in over output 0.55s to 1.2s and leaves over the last
        // third of the clip, which the speed change puts at the end of it.
        // The export evaluates that per frame from the clip's own bounds.
        let opening: Vec<_> = ["0.7", "0.9", "1.1"]
          .iter()
          .map(|time| exported_yellow(&destination, time, width, height))
          .collect();
        let closing: Vec<_> = ["1.25", "1.35"]
          .iter()
          .map(|time| exported_yellow(&destination, time, width, height))
          .collect();
        assert!(
          opening[0] < opening[1] && opening[1] < opening[2],
          "the arrow does not draw itself in: {opening:?}"
        );
        assert!(
          opening[2] > closing[0] && closing[0] > closing[1] && closing[1] > 0,
          "the arrow does not leave: {opening:?} then {closing:?}"
        );
      }
    }
  }
  std::fs::remove_dir_all(directory).unwrap();
}
