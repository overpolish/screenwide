// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use super::geometry::Rng;
use super::model::PinKeyframe;
use super::smooth::{smooth, Sample, POSITION};
use super::target::PinTarget;
use super::{track_pin, FrameSource, LumaFrame, PinRequest, PinnedPath};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const FRAME_MS: f64 = 1000.0 / 60.0;

/// A tall page of text-like lines and a few pictures: the sharp, dense,
/// partly repetitive content a scroll moves.
struct Page {
  width: usize,
  height: usize,
  pixels: Vec<f32>,
}

impl Page {
  fn new(width: usize, height: usize, seed: u32) -> Self {
    let mut pixels = vec![236.0_f32; width * height];
    let mut rng = Rng::new(seed);
    let mut y = 12;
    while y + 14 < height {
      if rng.below(7) == 0 {
        // A picture: a smooth gradient with a few blobs.
        let (x0, w, h) = (
          8 + rng.below(width / 2),
          40 + rng.below(80),
          40 + rng.below(40),
        );
        let tone = rng.below(120) as f32;
        for py in y..(y + h).min(height) {
          for px in x0..(x0 + w).min(width) {
            let blob = ((px * 7 + py * 13 + rng.below(3)) % 29) as f32;
            pixels[py * width + px] = 60.0 + tone + (py - y) as f32 + blob;
          }
        }
        y += h + 10;
        continue;
      }
      let mut x = 8 + rng.below(12);
      while x + 10 < width - 8 {
        let glyphs = 2 + rng.below(8);
        for _ in 0..glyphs {
          if x + 8 >= width - 8 {
            break;
          }
          for _ in 0..(2 + rng.below(3)) {
            let (sx, sy) = (x + rng.below(6), y + rng.below(9));
            let (dx, dy) = if rng.below(2) == 0 {
              (1, 3 + rng.below(6))
            } else {
              (2 + rng.below(4), 1)
            };
            for py in sy..(sy + dy).min(height) {
              for px in sx..(sx + dx).min(width) {
                pixels[py * width + px] = 30.0;
              }
            }
          }
          x += 7;
        }
        x += 5 + rng.below(4);
      }
      y += 18;
    }
    Self {
      width,
      height,
      pixels,
    }
  }

  /// A `WIDTH` by `HEIGHT` view whose top-left is at `(x, y)` on the page,
  /// sampled bilinearly so a movement can land between pixels.
  fn view(&self, ms: u64, x: f64, y: f64) -> LumaFrame {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    let at = |px: isize, py: isize| {
      if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {
        236.0
      } else {
        self.pixels[py as usize * self.width + px as usize]
      }
    };
    for row in 0..HEIGHT {
      for column in 0..WIDTH {
        let (sx, sy) = (x + f64::from(column), y + f64::from(row));
        let (fx, fy) = (sx.floor(), sy.floor());
        let (ax, ay) = ((sx - fx) as f32, (sy - fy) as f32);
        let (ix, iy) = (fx as isize, fy as isize);
        let value = (1.0 - ax) * (1.0 - ay) * at(ix, iy)
          + ax * (1.0 - ay) * at(ix + 1, iy)
          + (1.0 - ax) * ay * at(ix, iy + 1)
          + ax * ay * at(ix + 1, iy + 1);
        pixels.push(value.round().clamp(0.0, 255.0) as u8);
      }
    }
    LumaFrame {
      ms,
      width: WIDTH,
      height: HEIGHT,
      pixels,
    }
  }
}

/// Decoded frames held in memory, served the way a decoder would.
struct Frames(Vec<LumaFrame>);

impl FrameSource for Frames {
  fn source_size(&self) -> (u32, u32) {
    (WIDTH, HEIGHT)
  }

  fn read(
    &mut self,
    start_ms: u64,
    end_ms: u64,
    each: &mut dyn FnMut(LumaFrame) -> bool,
  ) -> Result<(), String> {
    for frame in self
      .0
      .iter()
      .filter(|frame| frame.ms >= start_ms && frame.ms < end_ms)
    {
      if !each(frame.clone()) {
        break;
      }
    }
    Ok(())
  }
}

fn ms(index: usize) -> u64 {
  (index as f64 * FRAME_MS).round() as u64
}

/// A scroll of the page: the view's top at each frame.
fn scrolled(page: &Page, tops: &[f64]) -> Frames {
  Frames(
    tops
      .iter()
      .enumerate()
      .map(|(index, top)| page.view(ms(index), 0.0, *top))
      .collect(),
  )
}

fn request(
  frames: &Frames,
  keyframes: &[(usize, [f64; 2])],
  region: [f64; 4],
  redaction: bool,
) -> PinRequest {
  let last = frames.0.last().expect("frames").ms;
  PinRequest {
    start_ms: 0,
    end_ms: last + 1,
    keyframes: keyframes
      .iter()
      .map(|&(index, [dx, dy])| PinKeyframe {
        ms: ms(index),
        dx,
        dy,
        edges: None,
        out_of_view: false,
      })
      .collect(),
    target: PinTarget {
      region,
      anchor: [0.5 * (region[0] + region[2]), 0.5 * (region[1] + region[3])],
      redaction,
    },
  }
}

fn pin(frames: &mut Frames, pinned: usize, region: [f64; 4], redaction: bool) -> PinnedPath {
  let request = request(frames, &[(pinned, [0.0; 2])], region, redaction);
  track_pin(frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped")
}

/// A momentum scroll: speeding up, coasting and easing to a stop.
fn momentum(frames: usize, peak: f64) -> Vec<f64> {
  let mut top = 0.0;
  (0..frames)
    .map(|index| {
      let t = index as f64 / (frames - 1) as f64;
      top += peak * (std::f64::consts::PI * t).sin().powi(2);
      top
    })
    .collect()
}

#[test]
fn a_scroll_is_followed_to_within_a_fraction_of_a_pixel() {
  let page = Page::new(WIDTH as usize, 2_400, 7);
  let tops = momentum(90, 11.3);
  let mut frames = scrolled(&page, &tops);
  // Pinned mid-scroll, so the path runs both ways from the pin.
  let pinned = 30;
  let path = pin(&mut frames, pinned, [100.0, 90.0, 160.0, 130.0], false);
  assert_eq!(path.samples.len(), tops.len());
  for (index, sample) in path.samples.iter().enumerate() {
    let expected = tops[pinned] - tops[index];
    let error = (f64::from(sample.dy) - expected)
      .abs()
      .max(f64::from(sample.dx).abs());
    // While the anchor is on the frame the path must be exact; once it has
    // gone the annotation is not drawn.
    if sample.visible {
      assert!(error < 0.3, "frame {index}: off by {error:.3} px");
    }
  }
}

#[test]
fn content_scrolled_off_the_top_and_back_is_hidden_then_found_again() {
  let page = Page::new(WIDTH as usize, 2_400, 11);
  // Down 400 pixels, a pause off screen, and back up to where it started.
  let mut tops = momentum(60, 13.4);
  let bottom = *tops.last().expect("tops");
  tops.extend(std::iter::repeat_n(bottom, 20));
  tops.extend(momentum(60, 13.4).iter().map(|travel| bottom - travel));
  let mut frames = scrolled(&page, &tops);
  let path = pin(&mut frames, 0, [120.0, 60.0, 180.0, 100.0], false);

  let anchor_y = 80.0;
  let mut hidden = 0;
  for (index, sample) in path.samples.iter().enumerate() {
    let on_screen = anchor_y - (tops[index] - tops[0]) >= 0.0;
    if !on_screen {
      assert!(
        !sample.visible,
        "frame {index} shows content that is off the frame"
      );
      hidden += 1;
    }
  }
  assert!(hidden > 20, "the content never left the frame");
  let end = path.samples.last().expect("samples");
  assert!(end.visible, "the content was not found again");
  let expected = tops[0] - tops[tops.len() - 1];
  assert!(
    (f64::from(end.dy) - expected).abs() < 0.5,
    "found again off by {}",
    end.dy
  );
}

#[test]
fn a_centre_weighted_box_follows_the_thing_inside_it_over_a_still_margin() {
  let background = Page::new(WIDTH as usize, HEIGHT as usize, 3);
  // Three lines of another page's text, inverted to stand apart.
  let object = Page::new(200, 200, 99);
  let mut frames = Vec::new();
  for index in 0..60 {
    // A 40 pixel picture drifting right and down over text that stays put.
    let (ox, oy) = (100.0 + 1.5 * index as f64, 90.0 + 0.5 * index as f64);
    let mut frame = background.view(ms(index), 0.0, 0.0);
    for y in 0..40 {
      for x in 0..40 {
        let (px, py) = (ox as usize + x, oy as usize + y);
        frame.pixels[py * WIDTH as usize + px] = 255 - object.pixels[(y + 10) * 200 + x + 20] as u8;
      }
    }
    frames.push(frame);
  }
  let mut frames = Frames(frames);
  // Drawn with a generous margin of the still text around the picture.
  let path = pin(&mut frames, 0, [80.0, 70.0, 160.0, 150.0], true);
  let last = path.samples.last().expect("samples");
  assert!(
    (last.dx - 1.5 * 59.0).abs() < 1.5,
    "followed {} across, not the picture",
    last.dx
  );
  assert!(
    (last.dy - 0.5 * 59.0).abs() < 1.5,
    "followed {} down, not the picture",
    last.dy
  );
}

#[test]
fn a_correction_is_met_exactly_and_joined_from_both_sides() {
  let page = Page::new(WIDTH as usize, 2_400, 5);
  let tops = momentum(90, 1.5);
  let mut frames = scrolled(&page, &tops);
  // Pinned at the first frame, and moved eight pixels right at frame 60.
  let request = request(
    &frames,
    &[(0, [0.0, 0.0]), (60, [8.0, tops[0] - tops[60]])],
    [100.0, 90.0, 160.0, 130.0],
    false,
  );
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  let at = |index: usize| path.samples[index];
  assert!(
    (at(60).dx - 8.0).abs() < 1e-3,
    "the keyframe is not met: {}",
    at(60).dx
  );
  assert!(at(0).dx.abs() < 1e-3, "the pin is not met: {}", at(0).dx);
  // After the correction the path follows the content from where it was put.
  for index in 61..90 {
    assert!(
      (at(index).dx - 8.0).abs() < 0.5,
      "frame {index}: {}",
      at(index).dx
    );
  }
  // Between the two it moves across, never back.
  for index in 1..60 {
    assert!(
      at(index).dx >= at(index - 1).dx - 0.1,
      "frame {index} turns back"
    );
  }
  assert!(
    (1.0..7.0).contains(&at(30).dx),
    "halfway is at {}",
    at(30).dx
  );
  // A correction settles the stretch it joins: nothing there is to check.
  assert!(path.weak.is_empty(), "flagged {:?}", path.weak);
}

#[test]
fn a_scroll_that_stops_dead_is_not_rounded_off_by_smoothing() {
  // Twenty pixels a frame, then nothing, measured cleanly.
  let samples: Vec<Sample> = (0..40)
    .map(|index| Sample {
      ms: ms(index),
      value: Some(((index.min(20) * 20) as f32, 1.0)),
    })
    .collect();
  let smoothed = smooth(&samples, POSITION);
  for (index, value) in smoothed.iter().enumerate() {
    let expected = (index.min(20) * 20) as f32;
    assert!(
      (value - expected).abs() < 0.2,
      "frame {index}: {value} for {expected}"
    );
  }
}

#[test]
fn a_stretch_without_a_match_is_bridged_between_its_ends() {
  let samples: Vec<Sample> = (0..30)
    .map(|index| Sample {
      ms: ms(index),
      value: (!(10..20).contains(&index)).then_some((index as f32 * 3.0, 1.0)),
    })
    .collect();
  let smoothed = smooth(&samples, POSITION);
  for (index, value) in smoothed.iter().enumerate().take(20).skip(10) {
    assert!(
      (value - index as f32 * 3.0).abs() < 0.5,
      "frame {index}: {value}"
    );
  }
}

#[test]
fn a_shorter_leg_cut_from_a_longer_one_is_the_leg_followed_on_its_own() {
  use super::leg::{track_leg, Leg};
  let page = Page::new(WIDTH as usize, 2_400, 13);
  let tops = momentum(90, 4.0);
  let mut frames = scrolled(&page, &tops);
  let target = PinTarget {
    region: [100.0, 90.0, 160.0, 130.0],
    anchor: [130.0, 110.0],
    redaction: false,
  };
  let from = PinKeyframe {
    ms: ms(45),
    dx: 0.0,
    dy: 0.0,
    edges: None,
    out_of_view: false,
  };
  let stop = AtomicBool::new(false);
  for forward in [true, false] {
    let leg = |until: usize| Leg {
      from,
      until_ms: ms(until),
      forward,
      target,
    };
    let (short, long) = if forward {
      (leg(60), leg(89))
    } else {
      (leg(30), leg(0))
    };
    assert_eq!(short.origin(), long.origin());
    assert!(short.within(long.until_ms));
    let followed =
      |leg: &Leg, frames: &mut Frames| track_leg(frames, leg, &stop, &mut || {}).unwrap().unwrap();
    let alone = followed(&short, &mut frames);
    let cut = short.cut_from(&followed(&long, &mut frames));
    assert_eq!(cut, alone, "forward: {forward}");
  }
}

/// Where the frames `split` makes change from one page to the other.
const SPLIT: usize = 160;

/// Frames whose left half is a page that stays put and whose right half is
/// another page scrolled to `tops`: a still sidebar beside a scrolling view.
fn split(tops: &[f64]) -> Frames {
  let still = Page::new(WIDTH as usize, HEIGHT as usize, 21);
  let moving = Page::new(WIDTH as usize, 2_400, 23);
  let width = WIDTH as usize;
  Frames(
    tops
      .iter()
      .enumerate()
      .map(|(index, top)| {
        let mut frame = still.view(ms(index), 0.0, 0.0);
        let scrolled = moving.view(ms(index), 0.0, *top);
        for row in 0..HEIGHT as usize {
          let span = row * width + SPLIT..(row + 1) * width;
          frame.pixels[span.clone()].copy_from_slice(&scrolled.pixels[span]);
        }
        frame
      })
      .collect(),
  )
}

#[test]
fn a_counter_and_a_text_box_follow_what_their_tips_point_at() {
  use crate::editor::annotations::counter::model::new_counter;
  use crate::editor::annotations::text::model::{new_text, TextPointer};
  use crate::editor::annotations::{AnnotationPoint, AnnotationShape};
  let point = |x, y| AnnotationPoint { x, y };
  let tops = momentum(60, 2.0);
  let mut frames = split(&tops);
  // Both sit mostly on the still half and point into the half that scrolls,
  // drawn at one source pixel per canvas pixel. The counter's disc is 40
  // pixels across each way, its tail tip 60 east of its centre.
  let mut counter = new_counter("c".to_owned(), point(150.0, 150.0), 1, None, Some(0.0));
  counter.style.width = 80.0;
  let mut text = new_text("t".to_owned(), point(0.0, 0.0), None, 1.0);
  text.style.width = 16.0;
  text.shape = AnnotationShape::Text {
    origin: point(20.0, 140.0),
    pointer: TextPointer {
      along: point(1.0, 0.0),
      reach: point(10.0, 0.0),
    },
    text: "Hi".to_owned(),
  };
  for annotation in [counter, text] {
    let target = PinTarget::of(&annotation, (WIDTH, HEIGHT), 1.0);
    assert!(
      target.region[0] >= SPLIT as f64,
      "the tip's patch {:?} is not over the scrolling half",
      target.region
    );
    let request = PinRequest {
      target,
      ..request(&frames, &[(0, [0.0; 2])], target.region, false)
    };
    let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
      .expect("tracks")
      .expect("not stopped");
    let last = path.samples.last().expect("samples");
    let expected = tops[0] - tops[tops.len() - 1];
    assert!(
      (f64::from(last.dy) - expected).abs() < 1.0 && last.dx.abs() < 1.0,
      "{:?} followed ({}, {}), not the scroll of {expected:.1}",
      annotation.shape.kind(),
      last.dx,
      last.dy
    );
  }
}

#[test]
fn a_counter_pointing_just_past_a_boundary_follows_what_lies_ahead() {
  use crate::editor::annotations::counter::model::new_counter;
  use crate::editor::annotations::AnnotationPoint;
  let tops = momentum(60, 2.0);
  let mut frames = split(&tops);
  // The tail tip stops five pixels short of the scrolling half, pointing
  // into it: most of what is around the tip itself is still.
  let mut counter = new_counter(
    "c".to_owned(),
    AnnotationPoint {
      x: SPLIT as f64 - 35.0,
      y: 120.0,
    },
    1,
    None,
    Some(0.0),
  );
  counter.style.width = 40.0;
  let target = PinTarget::of(&counter, (WIDTH, HEIGHT), 1.0);
  let request = PinRequest {
    target,
    ..request(&frames, &[(0, [0.0; 2])], target.region, false)
  };
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  // The tip itself sits on the still side, away from the points followed,
  // so it is carried a little short; a patch centred on the tip is carried
  // less than half the way.
  let last = path.samples.last().expect("samples");
  let expected = tops[0] - tops[tops.len() - 1];
  assert!(
    (f64::from(last.dy) - expected).abs() < 0.1 * expected.abs(),
    "followed {} rather than the scroll of {expected:.1}",
    last.dy
  );
}

#[test]
fn a_redaction_resized_at_a_keyframe_follows_what_the_resized_box_covers() {
  let tops = momentum(60, 2.0);
  let mut frames = split(&tops);
  // Drawn over the still half, then resized at its keyframe to lie over the
  // half that scrolls instead.
  let drawn = [60.0, 100.0, 140.0, 140.0];
  let mut request = request(&frames, &[(0, [0.0; 2])], drawn, true);
  request.keyframes[0].edges = Some([-(SPLIT as f64 + 20.0 - 60.0), 0.0, 120.0, 0.0]);
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  let last = path.samples.last().expect("samples");
  let expected = tops[0] - tops[tops.len() - 1];
  assert!(
    (f64::from(last.dy) - expected).abs() < 1.0,
    "followed {} rather than the scroll of {expected:.1}",
    last.dy
  );
}

/// Content going out of sight under something that stays put.
mod covered;
