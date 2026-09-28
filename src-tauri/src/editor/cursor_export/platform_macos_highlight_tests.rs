// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Highlights through a real Metal dispatch: the still compositor and the
//! video export, over light pages and dark ones.

use super::*;
use crate::editor::annotations::arrow::new_arrow;
use crate::editor::annotations::highlight::model::{new_highlight, HighlightBand, HighlightTone};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use crate::screenshots::CapturedImage;
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 160;
const YELLOW: [u8; 3] = [0xff, 0xcc, 0x00];
/// Two lines of "text": bars of ink, a gap between each word.
const LINES: [(u32, u32); 2] = [(40, 60), (90, 110)];

fn is_text(x: u32, y: u32) -> bool {
  LINES
    .iter()
    .any(|(top, bottom)| (*top..*bottom).contains(&y))
    && (40..280).contains(&x)
    && x % 24 < 16
}

/// A page in `surface` with bars of `ink` on it.
fn page(surface: [u8; 3], ink: [u8; 3]) -> CapturedImage {
  let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
  for y in 0..HEIGHT {
    for x in 0..WIDTH {
      let [r, g, b] = if is_text(x, y) { ink } else { surface };
      rgba.extend_from_slice(&[r, g, b, 255]);
    }
  }
  CapturedImage {
    rgba,
    width: WIDTH,
    height: HEIGHT,
  }
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// Both lines highlighted in yellow over a page of `tone`.
fn highlight(tone: HighlightTone, hand_drawn: bool) -> Annotation {
  let mut annotation = new_highlight("highlight".to_owned(), point(36.0, 50.0), None, 1.0);
  annotation.style.hand_drawn = hand_drawn;
  if let AnnotationShape::Highlight {
    end,
    bands,
    tone: held,
    ..
  } = &mut annotation.shape
  {
    *end = point(284.0, 100.0);
    *bands = LINES
      .iter()
      .map(|(top, bottom)| HighlightBand {
        left: 36.0,
        top: f64::from(*top) - 4.0,
        right: 284.0,
        bottom: f64::from(*bottom) + 4.0,
      })
      .collect();
    *held = tone;
  }
  annotation
}

fn composed(source: &CapturedImage, annotations: Vec<Annotation>) -> CapturedImage {
  let mut settings = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  settings.background_color = "#000000".to_owned();
  settings.background_type = "solid".to_owned();
  settings.crop_width = f64::from(WIDTH);
  settings.crop_height = f64::from(HEIGHT);
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = f64::from(WIDTH);
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.annotations = annotations;
  let rgba = crate::screenshots::compose_output_layers(
    source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba;
  let image = CapturedImage {
    rgba,
    width: WIDTH,
    height: HEIGHT,
  };
  if let Ok(directory) = std::env::var("SCREENWIDE_HIGHLIGHT_PNG_DIR") {
    let name = format!(
      "{}.png",
      std::thread::current().name().unwrap_or("highlight")
    );
    std::fs::create_dir_all(&directory).unwrap();
    let file = std::path::Path::new(&directory).join(name.replace("::", "-"));
    std::fs::write(file, crate::screenshots::encode_png(&image).unwrap()).unwrap();
  }
  image
}

fn at(image: &CapturedImage, x: u32, y: u32) -> [u8; 3] {
  let index = ((y * image.width + x) * 4) as usize;
  [
    image.rgba[index],
    image.rgba[index + 1],
    image.rgba[index + 2],
  ]
}

fn near(actual: [u8; 3], expected: [u8; 3], within: u8) -> bool {
  actual
    .iter()
    .zip(expected)
    .all(|(a, b)| a.abs_diff(b) <= within)
}

fn luminance(rgb: [u8; 3]) -> f64 {
  (0.2126 * f64::from(rgb[0]) + 0.7152 * f64::from(rgb[1]) + 0.0722 * f64::from(rgb[2])) / 255.0
}

#[test]
fn a_dark_page_turns_the_highlights_colour_with_its_text_inked_dark() {
  let source = page([24, 26, 30], [235, 235, 235]);
  let image = composed(
    &source,
    vec![highlight(
      HighlightTone {
        surface: 0.1,
        ink: 0.92,
      },
      false,
    )],
  );
  // Between words the page is now the highlight itself, not a dimmed tint.
  assert!(
    near(at(&image, 64, 50), YELLOW, 6),
    "{:?}",
    at(&image, 64, 50)
  );
  // The light text reads dark on it.
  assert!(
    luminance(at(&image, 50, 50)) < 0.25,
    "{:?}",
    at(&image, 50, 50)
  );
  // Outside the bands nothing changed.
  assert!(
    near(at(&image, 44, 130), [24, 26, 30], 2),
    "{:?}",
    at(&image, 44, 130)
  );
}

#[test]
fn a_light_page_turns_the_highlights_colour_and_keeps_its_text_dark() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let image = composed(
    &source,
    vec![highlight(
      HighlightTone {
        surface: 0.98,
        ink: 0.08,
      },
      false,
    )],
  );
  assert!(
    near(at(&image, 64, 100), YELLOW, 6),
    "{:?}",
    at(&image, 64, 100)
  );
  assert!(
    luminance(at(&image, 50, 100)) < 0.15,
    "{:?}",
    at(&image, 50, 100)
  );
}

#[test]
fn a_tint_marks_light_like_a_highlighter_and_lifts_dark() {
  // A light page takes the colour, and the text on it stays well darker than
  // the colour around it.
  let light = composed(
    &page([250, 250, 248], [20, 20, 20]),
    vec![highlight(HighlightTone::UNREAD, false)],
  );
  let (page_at, text_at) = (at(&light, 64, 50), at(&light, 48, 50));
  // The page takes the highlight's hue: red and green kept, blue well down.
  assert!(
    page_at[0] > 220 && page_at[1] > 180 && page_at[2] < page_at[0] - 120,
    "{page_at:?}"
  );
  assert!(
    luminance(text_at) < luminance(page_at) - 0.3,
    "{text_at:?} on {page_at:?}"
  );
  // A dark page is lifted enough to show where the box is, not painted over.
  let dark = composed(
    &page([24, 26, 30], [235, 235, 235]),
    vec![highlight(HighlightTone::UNREAD, false)],
  );
  let lifted = at(&dark, 64, 50);
  assert!(
    luminance(lifted) > luminance([24, 26, 30]) + 0.1 && luminance(lifted) < 0.4,
    "{lifted:?}"
  );
}

#[test]
fn a_highlight_half_drawn_in_leads_with_its_first_line() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let mut annotation = highlight(HighlightTone::default(), false);
  annotation.reveal = AnnotationReveal {
    low: 0.0,
    high: 0.45,
    ..AnnotationReveal::WHOLE
  };
  let image = composed(&source, vec![annotation]);
  assert!(
    near(at(&image, 64, 50), YELLOW, 6),
    "{:?}",
    at(&image, 64, 50)
  );
  // The first line is not finished yet at its far end; the second has been
  // started a little after it and is further behind.
  assert!(
    near(at(&image, 262, 50), [250, 250, 248], 2),
    "{:?}",
    at(&image, 262, 50)
  );
  let (first, second) = (reach(&image, 50), reach(&image, 100));
  assert!(second > 36 && second < first, "{first} {second}");
}

/// A box from (36, 10) to (284, 150) laid in clean strokes 20 tall.
fn clean_box() -> Annotation {
  let mut annotation = new_highlight("box".to_owned(), point(36.0, 10.0), None, 1.0);
  if let AnnotationShape::Highlight { end, bands, .. } = &mut annotation.shape {
    *end = point(284.0, 150.0);
    *bands = crate::editor::annotations::highlight::manual::strokes(point(36.0, 10.0), *end, 20.0);
    assert!(bands.len() >= 6, "{bands:?}");
  }
  annotation.style.manual = true;
  annotation
}

#[test]
fn a_clean_box_has_no_notch_where_two_strokes_meet() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let image = composed(&source, vec![clean_box()]);
  // Just inside the box's left side, where the first two strokes meet.
  assert!(
    near(at(&image, 36, 28), YELLOW, 12),
    "{:?}",
    at(&image, 36, 28)
  );
}

#[test]
fn a_box_of_many_strokes_draws_each_at_a_hands_pace() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let mut annotation = clean_box();
  annotation.reveal = AnnotationReveal {
    low: 0.0,
    high: 0.2,
    ..AnnotationReveal::WHOLE
  };
  let image = composed(&source, vec![annotation]);
  // A fifth of the way through, the first stroke is still going down rather
  // than done, however many strokes the box takes.
  let first = reach(&image, 16);
  assert!(first > 80 && first < 250, "{first}");
}

/// The furthest right the highlight reaches along row `y`.
fn reach(image: &CapturedImage, y: u32) -> u32 {
  (0..WIDTH)
    .rev()
    .find(|&x| near(at(image, x, y), YELLOW, 40))
    .unwrap_or(0)
}

#[test]
fn a_highlight_lands_on_its_end_rather_than_jumping_out_to_it() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let frame = |high: f32| {
    let mut annotation = highlight(HighlightTone::default(), false);
    annotation.reveal = AnnotationReveal {
      low: 0.0,
      high,
      ..AnnotationReveal::WHOLE
    };
    composed(&source, vec![annotation])
  };
  // Near the top of the last line's band, where a round tip and a square end
  // differ most. The last few pixels of the draw move the end by a few pixels.
  let (almost, landed) = (frame(0.995), frame(1.0));
  let (from, to) = (reach(&almost, 88), reach(&landed, 88));
  assert!(to >= from && to - from <= 4, "{from} -> {to}");
}

#[test]
fn a_hand_drawn_highlight_stays_on_its_lines() {
  let source = page([24, 26, 30], [235, 235, 235]);
  let image = composed(
    &source,
    vec![highlight(
      HighlightTone {
        surface: 0.1,
        ink: 0.92,
      },
      true,
    )],
  );
  // The middle of a band is marked however the stroke wobbles.
  let marked = at(&image, 160, 50);
  assert!(
    luminance(marked) > 0.5 || luminance(marked) < 0.25,
    "{marked:?}"
  );
  assert!(!near(at(&image, 160, 50), [24, 26, 30], 2));
  // Well clear of both lines the page is untouched.
  assert!(
    near(at(&image, 160, 76), [24, 26, 30], 2),
    "{:?}",
    at(&image, 160, 76)
  );
  assert!(
    near(at(&image, 8, 50), [24, 26, 30], 2),
    "{:?}",
    at(&image, 8, 50)
  );
}

#[test]
fn an_arrow_is_drawn_over_a_highlight_rather_than_recoloured_by_it() {
  let source = page([250, 250, 248], [20, 20, 20]);
  let mut arrow = new_arrow(
    "arrow".to_owned(),
    point(20.0, 50.0),
    point(300.0, 50.0),
    None,
  );
  arrow.style.color = "#0000ff".to_owned();
  // The arrow comes first in the list, and still lies on top.
  let image = composed(
    &source,
    vec![arrow, highlight(HighlightTone::default(), false)],
  );
  assert!(
    near(at(&image, 160, 50), [0, 0, 255], 4),
    "{:?}",
    at(&image, 160, 50)
  );
}

#[test]
fn an_exported_video_carries_the_highlight_on_both_planes() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-highlight-video-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("dark.mov");
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i"
    ])
    .arg(format!("color=c=0x181a1e:s={WIDTH}x{HEIGHT}:r=10:d=1"))
    .args([
      "-vf",
      "drawbox=x=40:y=40:w=16:h=20:color=white:t=fill,format=yuv420p"
    ])
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
    .arg(&source)
    .status()
    .unwrap()
    .success());
  let clip = RecordingAnnotationClip {
    pin: None,
    annotation: Annotation {
      animated: false,
      ..highlight(
        HighlightTone {
          surface: 0.1,
          ink: 0.95,
        },
        false,
      )
    },
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 1_000,
  };
  let destination = directory.join("highlighted.mp4");
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      annotation_clips: vec![clip],
      segments: vec![RecordingTimelineSegment {
        id: 0,
        source_start: 0.0,
        source_end: 1.0,
        playback_rate: 1.0,
      }],
    },
    1_000,
  );
  let mut output = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  output.background_color = "#000000".into();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.crop_width = f64::from(WIDTH);
  output.crop_height = f64::from(HEIGHT);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(WIDTH);
  output.image_x = 0.0;
  output.image_y = 0.0;
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: AnnotationTrack::Primary,
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
    duration_ms: 1_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output,
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: timeline.as_ref(),
    video: VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: WIDTH,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
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
    .unwrap()
    .stdout;
  let rgb = |x: u32, y: u32| {
    let index = ((y * WIDTH + x) * 3) as usize;
    [frame[index], frame[index + 1], frame[index + 2]]
  };
  assert!(near(rgb(120, 50), YELLOW, 24), "{:?}", rgb(120, 50));
  assert!(luminance(rgb(48, 50)) < 0.3, "{:?}", rgb(48, 50));
  assert!(luminance(rgb(120, 140)) < 0.15, "{:?}", rgb(120, 140));
  std::fs::remove_dir_all(directory).unwrap();
}
