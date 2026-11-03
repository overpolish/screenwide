// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Export island's compare, made from real frames: a 3600 by 2338
//! recording drawn at Original and at the smaller looks by the export's
//! compositor, the exported frame laid over the Original one by the canvas
//! both show. Bands across the top, the middle and the bottom of the picture
//! land on the same rows on both sides.

use std::path::Path;
use std::process::Command;

use super::*;
use crate::media_preview::{FrameFit, PreviewPicture};

const WIDTH: u32 = 3_600;
const HEIGHT: u32 = 2_338;
/// The first row and the height of each white band in the recording.
const BANDS: [(u32, u32); 3] = [(8, 6), (1_166, 6), (2_324, 6)];

/// The luma of every pixel of the PNG at `path`, and its size.
fn luma(path: &Path) -> (Vec<f64>, u32, u32) {
  let picture = image::open(path).unwrap().to_luma8();
  let (width, height) = picture.dimensions();
  (
    picture.into_raw().into_iter().map(f64::from).collect(),
    width,
    height,
  )
}

/// The brightness-weighted middle row of what is lit within `reach` rows of
/// row `around`, in the middle half of the columns of a `width` wide
/// picture.
fn lit_row(pixels: &[f64], width: u32, around: f64, reach: f64) -> f64 {
  let (mut weight, mut sum) = (0.0, 0.0);
  let rows = pixels.len() as f64 / f64::from(width);
  for row in ((around - reach).max(0.0) as u32)..((around + reach).min(rows) as u32) {
    let start = (row * width + width / 4) as usize;
    let lit: f64 = pixels[start..start + (width / 2) as usize]
      .iter()
      .map(|value| (value - 16.0).max(0.0))
      .sum();
    weight += lit;
    sum += lit * f64::from(row);
  }
  assert!(weight > 0.0, "no band lit there");
  sum / weight
}

/// The frame at `scale` percent of the recording `source` composed with
/// `output`, as the compositor draws it, at `png`.
fn drawn(
  source: &Path,
  output: &screenwide_output::ScreenshotOutputSettings,
  scale: u16,
  png: &Path,
) {
  let cancelled = AtomicBool::new(false);
  let result = preview_frame(
    CursorExportRequest {
      camera_effects: None,
      annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
      audio_layout: AudioLayout::SeparateTracks,
      audio_source: None,
      camera: None,
      camera_on_top: true,
      cancelled: &cancelled,
      captions: None,
      cursor: None,
      cursor_effects: CursorEffectSettings::default(),
      keyboard: None,
      keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings::default(),
      destination: Path::new(""),
      duration_ms: 500,
      height: HEIGHT,
      on_progress: &mut |_| {},
      output,
      screen: source,
      selection: &TrackSelection::default(),
      timeline: None,
      video: VideoExportOptions {
        compression: 0,
        resolution_scale_percent: scale,
        source_scale_percent: 100,
      },
      width: WIDTH,
    },
    0,
    PreviewPicture::Drawn,
    png,
  )
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
}

/// The compare as the island shows it: the Original frame on the left of
/// the middle, the exported one laid over it by the canvas on the right,
/// sampled bilinearly in the Original's pixels.
fn compared(original: &Path, exported: &Path, fit: FrameFit, compare: &Path) {
  let original = image::open(original).unwrap().to_rgb8();
  let exported = image::open(exported).unwrap().to_rgb8();
  let mut shown = original.clone();
  let sample = |x: f64, y: f64, channel: usize| {
    let x = x.clamp(0.0, f64::from(exported.width() - 1));
    let y = y.clamp(0.0, f64::from(exported.height() - 1));
    let (left, top) = (x.floor() as u32, y.floor() as u32);
    let (right, bottom) = (
      (left + 1).min(exported.width() - 1),
      (top + 1).min(exported.height() - 1),
    );
    let (dx, dy) = (x - f64::from(left), y - f64::from(top));
    let at = |x, y| f64::from(exported.get_pixel(x, y)[channel]);
    let upper = at(left, top) * (1.0 - dx) + at(right, top) * dx;
    let lower = at(left, bottom) * (1.0 - dx) + at(right, bottom) * dx;
    upper * (1.0 - dy) + lower * dy
  };
  for (x, y, pixel) in shown
    .enumerate_pixels_mut()
    .filter(|(x, ..)| *x >= WIDTH / 2)
  {
    let from_x = fit.x(f64::from(x) + 0.5) - 0.5;
    let from_y = fit.y(f64::from(y) + 0.5) - 0.5;
    for channel in 0..3 {
      pixel[channel] = sample(from_x, from_y, channel).round() as u8;
    }
  }
  shown.save(compare).unwrap();
}

#[test]
fn lines_up_the_compared_frames_at_the_smaller_looks() {
  let directory = std::env::temp_dir().join("screenwide-compare-fit");
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let bands = BANDS
    .iter()
    .map(|(top, height)| format!("drawbox=x=0:y={top}:w=iw:h={height}:color=white:t=fill"))
    .collect::<Vec<_>>()
    .join(",");
  let status = Command::new(screenwide_sidecar::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
    ])
    .arg(format!("color=c=black:s={WIDTH}x{HEIGHT}:r=30:d=0.5"))
    // Near lossless rather than lossless: x264 writes lossless video in the
    // High 4:4:4 Predictive profile, which Media Foundation cannot decode.
    .args([
      "-vf", &bands, "-c:v", "libx264", "-qp", "1", "-pix_fmt", "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap();
  assert!(status.success());

  // The picture as recorded, and inset in a padded frame whose placement
  // falls between pixels once scaled.
  let mut plain = screenwide_output::fixtures::test_output_settings(WIDTH, HEIGHT);
  plain.background_color = "#000000".to_owned();
  plain.drop_shadow = false;
  plain.radius_percent = 0.0;
  (plain.crop_x, plain.crop_y) = (0.0, 0.0);
  (plain.crop_width, plain.crop_height) = (f64::from(WIDTH), f64::from(HEIGHT));
  (plain.image_x, plain.image_y, plain.image_width) = (0.0, 0.0, f64::from(WIDTH));
  let mut inset = plain.clone();
  let inset_height = 3_000.0 * f64::from(HEIGHT) / f64::from(WIDTH);
  (inset.image_x, inset.image_y, inset.image_width) = (300.0, 195.0, 3_000.0);
  (inset.crop_x, inset.crop_y) = (300.0, 195.0);
  (inset.crop_width, inset.crop_height) = (3_000.0, inset_height);

  for (name, output) in [("plain", &plain), ("inset", &inset)] {
    let original = directory.join(format!("{name}-original.png"));
    drawn(&source, output, 100, &original);
    let (pixels, width, height) = luma(&original);
    assert_eq!((width, height), (WIDTH, HEIGHT));
    let original_rows = BANDS
      .iter()
      .map(|(top, band)| {
        let canvas =
          |row: u32| output.image_y + f64::from(row) * output.image_width / f64::from(WIDTH);
        let middle = (canvas(*top) + canvas(top + band)) / 2.0;
        lit_row(&pixels, width, middle, 12.0)
      })
      .collect::<Vec<_>>();
    for scale in [75, 50] {
      let fit = FrameFit::of(
        WIDTH,
        HEIGHT,
        VideoExportOptions {
          compression: 0,
          resolution_scale_percent: scale,
          source_scale_percent: 100,
        },
      );
      let exported = directory.join(format!("{name}-{scale}.png"));
      drawn(&source, output, scale, &exported);
      let (pixels, width, height) = luma(&exported);
      assert_eq!((width, height), (fit.width, fit.height));
      compared(
        &original,
        &exported,
        fit,
        &directory.join(format!("{name}-{scale}-compare.png")),
      );
      for (row, expected) in original_rows.iter().enumerate() {
        let around = fit.y(*expected + 0.5) - 0.5;
        let found = lit_row(&pixels, width, around, 8.0);
        // Carried back by the canvas the compare lays over the Original's.
        let [_, top, _, canvas_height] = fit.canvas();
        let shown = (found + 0.5 - top) * f64::from(HEIGHT) / canvas_height - 0.5;
        println!(
          "{name} at {scale}%: band {row} on Original row {expected:.3}, exported row {found:.3} \
           of {height}, compared row {shown:.3}, off by {:.3}",
          shown - expected
        );
        assert!(
          (shown - expected).abs() <= 0.5,
          "{name} at {scale}%: band {row} shows at {shown}, not {expected}"
        );
      }
    }
  }
  // Kept, to be looked at, where asked for.
  if std::env::var_os("SCREENWIDE_KEEP_COMPARE_FRAMES").is_none() {
    let _ = std::fs::remove_dir_all(directory);
  }
}
