// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Cleanup;

/// A 1080-line matte: the sizes the rules are tuned in apply one to one.
const SIZE: (usize, usize) = (480, 1080);

/// A matte and a frame of `SIZE`, a person filling the lower half, each
/// pixel's alpha and colour from `paint`.
fn frame(paint: impl Fn(usize, usize) -> (u8, [u8; 3])) -> (Vec<u8>, Vec<u8>) {
  let (mut alpha, mut rgb) = (Vec::new(), Vec::new());
  for y in 0..SIZE.1 {
    for x in 0..SIZE.0 {
      let (a, colour) = paint(x, y);
      alpha.push(a);
      rgb.extend(colour);
    }
  }
  (alpha, rgb)
}

const WALL: [u8; 3] = [230, 230, 225];
const SKIN: [u8; 3] = [200, 150, 120];

fn person(x: usize, y: usize) -> bool {
  let _ = x;
  y >= 540
}

fn cleaned(paint: impl Fn(usize, usize) -> (u8, [u8; 3])) -> impl Fn(usize, usize) -> u8 {
  let (mut alpha, rgb) = frame(paint);
  Cleanup::new(SIZE).apply(&mut alpha, &rgb);
  move |x, y| alpha[y * SIZE.0 + x]
}

#[test]
fn the_room_and_the_person_are_decided_away_from_their_edge() {
  // RVM's haze over the room, and a half-sure patch on the person.
  let matte = cleaned(|x, y| {
    let haze = (x / 40 + y / 40) % 2 == 0;
    match (person(x, y), haze) {
      (true, _) if (300..340).contains(&x) && (800..840).contains(&y) => (150, SKIN),
      (true, _) => (255, SKIN),
      (false, true) => (60, WALL),
      (false, false) => (0, WALL),
    }
  });
  assert_eq!(matte(240, 200), 0, "haze in the room");
  assert_eq!(matte(320, 820), 255, "a half-sure patch inside the person");
  assert_eq!(matte(240, 900), 255);
}

#[test]
fn the_edge_fades_once_across_the_line() {
  let matte = cleaned(|x, y| {
    (
      if person(x, y) { 255 } else { 0 },
      if person(x, y) { SKIN } else { WALL },
    )
  });
  let column = (500..580).map(|y| matte(240, y)).collect::<Vec<_>>();
  assert!(
    column.windows(2).all(|pair| pair[0] <= pair[1]),
    "{column:?}"
  );
  let soft = column
    .iter()
    .filter(|&&value| value > 5 && value < 250)
    .count();
  assert!((6..=24).contains(&soft), "fades over {soft} pixels");
}

#[test]
fn specks_go_pinholes_fill_and_real_gaps_stay() {
  let matte = cleaned(|x, y| {
    let speck = (100..106).contains(&x) && (100..106).contains(&y);
    let pinhole = (100..106).contains(&x) && (900..906).contains(&y);
    // A gap closed in by the person on every side, as between an arm and
    // the body, wide enough to be more than a pinhole.
    let gap = (300..400).contains(&x) && (700..800).contains(&y);
    match (person(x, y), speck, pinhole, gap) {
      (false, true, ..) => (255, SKIN),
      (true, _, true, _) | (true, .., true) => (0, WALL),
      (true, ..) => (255, SKIN),
      _ => (0, WALL),
    }
  });
  assert_eq!(matte(103, 103), 0, "a speck of person in the room");
  assert_eq!(matte(103, 903), 255, "a pinhole in the person");
  assert_eq!(matte(350, 750), 0, "a gap the person closes round");
}

#[test]
fn a_bite_coloured_like_the_person_is_kept_and_one_like_the_room_is_not() {
  // Two bites out of the person's outline RVM was unsure of: one the colour
  // of the person, as where it lost a patch against a sofa, one the wall's.
  let matte = cleaned(|x, y| {
    let bite = (540..570).contains(&y);
    let skin_bite = bite && (100..116).contains(&x);
    let wall_bite = bite && (300..316).contains(&x);
    match (person(x, y), skin_bite, wall_bite) {
      (true, true, _) => (20, SKIN),
      (true, _, true) => (20, WALL),
      (true, ..) => (255, SKIN),
      _ => (0, WALL),
    }
  });
  assert!(
    matte(108, 558) > 200,
    "the skin-coloured bite: {}",
    matte(108, 558)
  );
  assert!(
    matte(308, 558) < 100,
    "the wall-coloured bite: {}",
    matte(308, 558)
  );
}
