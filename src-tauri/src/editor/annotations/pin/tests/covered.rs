// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn content_that_scrolls_under_a_still_header_is_hidden_until_it_comes_back() {
  use crate::editor::annotations::counter::model::new_counter;
  use crate::editor::annotations::AnnotationPoint;
  let (tops, mut frames) = under_header();
  // A counter below the content its tail points up at, pinned on the first
  // frame and put right, six pixels lower, once the content is back.
  let mut counter = new_counter(
    "c".to_owned(),
    AnnotationPoint { x: 160.0, y: 170.0 },
    1,
    None,
    Some(-std::f64::consts::FRAC_PI_2),
  );
  counter.style.width = 40.0;
  let target = PinTarget::of(&counter, (WIDTH, HEIGHT), 1.0);
  let last = tops.len() - 1;
  let request = PinRequest {
    target,
    ..request(
      &frames,
      &[(0, [0.0; 2]), (last, [0.0, 6.0])],
      target.region,
      false,
    )
  };
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  for (index, sample) in path.samples.iter().enumerate() {
    let content = tops[0] - tops[index];
    // Before it went, the correction made after it came back has no say.
    if index < 20 {
      assert!(
        sample.visible && f64::from(sample.dy).abs() < 0.5,
        "frame {index} drifted to {} while still",
        sample.dy
      );
    }
    // While what the tail points at is plainly on screen it is followed.
    // The patch reaches ahead of the tip, into the header, so the tip is
    // given up a few frames before it goes under itself.
    if index < 27 {
      assert!(
        sample.visible && (f64::from(sample.dy) - content).abs() < 1.5,
        "frame {index}: {} against {content}",
        sample.dy
      );
    }
    // Held under the header it is not shown pointing at the header.
    if (45..55).contains(&index) {
      assert!(!sample.visible, "frame {index} is shown while covered");
    }
  }
  // "Show Here" has the stretch it was hidden for to bring it back in.
  assert!(
    path
      .covered
      .iter()
      .any(|[start, end]| (*start..*end).contains(&ms(50))),
    "{:?}",
    path.covered
  );
  assert!(path.samples[last].visible, "not shown once back");
}

/// A page under a still header band 50 pixels tall: still for 20 frames,
/// scrolled 150 pixels up under the header over 20, held there for 20,
/// scrolled back down over 20 and still for the last 20.
fn under_header() -> (Vec<f64>, Frames) {
  let mut tops = vec![0.0; 20];
  tops.extend((1..=20).map(|index| 7.5 * index as f64));
  tops.extend(std::iter::repeat_n(150.0, 20));
  tops.extend((1..=20).map(|index| 150.0 - 7.5 * index as f64));
  tops.extend(std::iter::repeat_n(0.0, 20));
  let frames = beneath_header(&tops);
  (tops, frames)
}

/// The page scrolled to `tops` under a still header band 50 pixels tall.
fn beneath_header(tops: &[f64]) -> Frames {
  const HEADER: usize = 50;
  let page = Page::new(WIDTH as usize, 2_400, 31);
  let header = Page::new(WIDTH as usize, HEIGHT as usize, 37).view(0, 0.0, 0.0);
  let mut frames = scrolled(&page, tops);
  for frame in &mut frames.0 {
    let rows = HEADER * WIDTH as usize;
    frame.pixels[..rows].copy_from_slice(&header.pixels[..rows]);
  }
  frames
}

#[test]
fn a_redaction_is_trimmed_clear_of_a_still_header_and_hidden_while_under_it() {
  const HEADER: f64 = 50.0;
  let (tops, mut frames) = under_header();
  // A box over a line of the page, which scrolls under the header and comes
  // back to where it left, pinned at both ends.
  let region = [100.0, 110.0, 220.0, 140.0];
  let last = tops.len() - 1;
  let request = request(&frames, &[(0, [0.0; 2]), (last, [0.0; 2])], region, true);
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  for (index, sample) in path.samples.iter().enumerate() {
    let content = tops[0] - tops[index];
    let line = [region[1] + content, region[3] + content];
    let [_, top, _, bottom] = path.views[index].map(f64::from);
    let drawn = [
      (region[1] + f64::from(sample.dy)).max(top),
      (region[3] + f64::from(sample.dy)).min(bottom),
    ];
    // Carried under the header at the speed it went, it may lag the line by
    // a pixel or two.
    if line[1] <= HEADER - 2.0 {
      assert!(
        !sample.visible,
        "frame {index} shown with its line under the header"
      );
      continue;
    }
    if line[1] <= HEADER {
      continue;
    }
    // What shows of the line stays hidden, and the header is left alone.
    assert!(
      sample.visible,
      "frame {index} hidden with its line in sight"
    );
    assert!(
      drawn[0] >= HEADER - 0.5,
      "frame {index} reaches up to {}",
      drawn[0]
    );
    assert!(
      drawn[0] <= line[0].max(HEADER) + 2.0 && drawn[1] >= line[1] - 2.0,
      "frame {index}: {drawn:?} against {line:?}"
    );
  }
  // Known to be under the header, it is not lost there, and can be brought
  // back in view by hand.
  assert!(path.weak.is_empty(), "{:?}", path.weak);
  assert!(
    path
      .under
      .iter()
      .any(|[start, end]| (*start..*end).contains(&ms(50))),
    "{:?}",
    path.under
  );
  assert_eq!(path.under, path.covered);
}

#[test]
fn a_redaction_lost_going_under_a_header_is_not_hidden_without_seeing_it_come_out() {
  // Scrolled three quarters under the header and left there, the line is
  // lost with a sliver still showing, so it is kept, grown and flagged for
  // checking.
  let mut tops = vec![0.0; 20];
  tops.extend((1..=11).map(|index| 7.5 * index as f64));
  tops.extend(std::iter::repeat_n(82.5, 40));
  let mut frames = beneath_header(&tops);
  let region = [100.0, 110.0, 220.0, 140.0];
  let request = request(&frames, &[(0, [0.0; 2])], region, true);
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  assert!(path.under.is_empty(), "{:?}", path.under);
  assert!(path.samples.iter().all(|sample| sample.visible));
  assert!(!path.weak.is_empty(), "never flagged");
}

#[test]
fn a_stretch_said_to_be_out_of_view_is_hidden_and_not_followed() {
  let page = Page::new(WIDTH as usize, 2_400, 41);
  let tops = momentum(90, 1.5);
  let mut frames = scrolled(&page, &tops);
  // Out of view by the hand from frame 30, back in view at frame 60, on a
  // page that could have been followed throughout.
  let mut request = request(
    &frames,
    &[(0, [0.0; 2]), (30, [0.0; 2]), (60, [0.0; 2])],
    [100.0, 90.0, 160.0, 130.0],
    true,
  );
  request.keyframes[1].out_of_view = true;
  request.keyframes[2].dy = tops[0] - tops[60];
  assert!(request.legs().iter().all(|leg| {
    let (low, high) = if leg.forward {
      (leg.from.ms, leg.until_ms)
    } else {
      (leg.until_ms, leg.from.ms)
    };
    high <= ms(30) || low >= ms(60)
  }));
  let path = track_pin(&mut frames, &request, &AtomicBool::new(false))
    .expect("tracks")
    .expect("not stopped");
  for sample in &path.samples {
    let out = (ms(30)..ms(60)).contains(&sample.ms);
    assert_eq!(sample.visible, !out, "at {} ms", sample.ms);
  }
  // Brought back in view there, and neither lost nor off the frame.
  assert!(path
    .covered
    .iter()
    .any(|[start, end]| *start <= ms(30) && *end >= ms(60)));
  assert!(path.weak.is_empty() && path.hidden.is_empty(), "{path:?}");
}

#[test]
fn content_said_to_be_back_in_view_is_found_near_where_it_was_last_shown() {
  use crate::editor::annotations::pin::find::find_again;
  let (tops, mut frames) = under_header();
  // A box no more than about four times as long as it is tall, which has a
  // look to match.
  let region = [100.0, 100.0, 200.0, 140.0];
  let request = request(&frames, &[(0, [0.0; 2])], region, true);
  let pinned = request.keyframes[0];
  // Last shown as it went under the header, 60 pixels up.
  let near = [0.0, -60.0];
  // Coming back down, 22 pixels lower than where it was last shown.
  let back = 75;
  let found = find_again(&mut frames, request.target, pinned, near, ms(back))
    .expect("reads")
    .expect("found");
  let content = tops[0] - tops[back];
  assert!(
    found[0].abs() < 1.0 && (found[1] - content).abs() < 1.0,
    "found at {found:?}, content at {content}"
  );
  // Still under the header, it is not found anywhere else.
  let under = find_again(&mut frames, request.target, pinned, near, ms(50)).expect("reads");
  assert!(under.is_none(), "found under the header at {under:?}");
}
