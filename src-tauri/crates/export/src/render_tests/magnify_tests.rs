// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's lens, through a real GPU dispatch: what it shows of the
//! picture under it, and how.

use screenwide_annotations::magnify::model::new_magnify;
use screenwide_annotations::redact::model::new_redact;
use screenwide_annotations::{Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape};

const SIZE: (u32, u32) = (320, 200);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A clear square lens 80 across centred at (220, 100), enlarging `zoom`
/// times, with no shadow to fall over what it shows.
fn lens(zoom: f64) -> Annotation {
  lens_at(point(220.0, 100.0), zoom)
}

/// A clear square lens 80 across centred at `centre`, enlarging `zoom` times.
fn lens_at(centre: AnnotationPoint, zoom: f64) -> Annotation {
  let mut annotation = new_magnify("magnify".to_owned(), centre, None, None);
  annotation.shape = AnnotationShape::Magnify {
    start: point(centre.x - 40.0, centre.y - 40.0),
    end: point(centre.x + 40.0, centre.y + 40.0),
    zoom,
  };
  annotation.style.radius = 0.0;
  annotation.style.width = 0.0;
  annotation.style.shadow = false;
  annotation
}

/// `source` composed as a screenshot the size of the canvas, with
/// `annotations` over it.
fn composed(source: Vec<u8>, annotations: Vec<Annotation>) -> Vec<u8> {
  composed_at(SIZE, source, annotations)
}

/// `source`, `size` pixels, composed as a screenshot the size of the canvas,
/// with `annotations` over it.
fn composed_at(size: (u32, u32), source: Vec<u8>, annotations: Vec<Annotation>) -> Vec<u8> {
  let source = screenwide_imaging::CapturedImage {
    rgba: source,
    width: size.0,
    height: size.1,
  };
  let mut settings = screenwide_output::fixtures::test_output_settings(size.0, size.1);
  settings.background_color = "#000000".to_owned();
  settings.background_type = "solid".to_owned();
  settings.crop_height = f64::from(size.1);
  settings.crop_width = f64::from(size.0);
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = f64::from(size.0);
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.annotations = annotations;
  screenwide_compositor::compositor::still_overlay::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba
}

/// Black and white in alternate columns.
fn stripes() -> Vec<u8> {
  (0..SIZE.1)
    .flat_map(|_| (0..SIZE.0).flat_map(|x| if x % 2 == 0 { [0, 0, 0, 255] } else { [255; 4] }))
    .collect()
}

fn red(rgba: &[u8], x: u32, y: u32) -> u8 {
  rgba[((y * SIZE.0 + x) * 4) as usize]
}

fn blue(rgba: &[u8], x: u32, y: u32) -> u8 {
  rgba[((y * SIZE.0 + x) * 4 + 2) as usize]
}

#[test]
fn an_enlarged_pixel_is_drawn_as_a_crisp_square() {
  let drawn = composed(stripes(), vec![lens(4.0)]);
  // Four times over, each source column is four output columns wide, and
  // only the one pixel across each edge may fall between black and white.
  let row: Vec<u8> = (200..240).map(|x| red(&drawn, x, 100)).collect();
  let between = row
    .iter()
    .filter(|&&level| (24..=231).contains(&level))
    .count();
  assert!(between <= row.len() / 4, "{row:?}");
  assert!(row.iter().any(|&level| level < 24) && row.iter().any(|&level| level > 231));
  // Each run of one level is the zoom wide, give or take the blended edge.
  let runs = row
    .windows(2)
    .filter(|pair| (pair[0] > 127) != (pair[1] > 127))
    .count();
  assert!((8..=12).contains(&runs), "{row:?}");
}

/// A bright square 20 across under the lens's middle, on a dark picture.
fn lit_middle() -> Vec<u8> {
  (0..SIZE.1)
    .flat_map(|y| {
      (0..SIZE.0).flat_map(move |x| {
        let lit = (210..230).contains(&x) && (90..110).contains(&y);
        if lit {
          [255; 4]
        } else {
          [20, 20, 20, 255]
        }
      })
    })
    .collect()
}

#[test]
fn the_lens_shows_what_lies_under_its_middle_enlarged() {
  let drawn = composed(lit_middle(), vec![lens(2.0)]);
  // Twice over, the 20 lit pixels under its middle fill its middle 40.
  assert!(red(&drawn, 220, 100) > 240);
  assert!(red(&drawn, 205, 85) > 240);
  assert!(red(&drawn, 190, 70) < 40);
  // Off the lens, the picture is as it was.
  assert!(red(&drawn, 120, 100) < 40);
}

#[test]
fn the_glass_catches_the_light_along_its_rim() {
  let drawn = composed(lit_middle(), vec![lens(2.0)]);
  // The lens's top edge faces the light: its outermost row is lit well
  // above the dark picture it shows, a row further in is not.
  assert!(red(&drawn, 200, 60) > 80, "{}", red(&drawn, 200, 60));
  assert!(red(&drawn, 200, 64) < 40, "{}", red(&drawn, 200, 64));
}

#[test]
fn a_lens_setting_off_is_flat() {
  // Not yet settled, the lens enlarges nothing and no glass bends what it
  // shows, so it lifts off the picture without a jump.
  let mut resting = lens(2.0);
  resting.reveal = screenwide_annotations::reveal::AnnotationReveal {
    scale: 0.0,
    opacity: 0.0,
    previous: [0.0, 1.0, 0.0, 0.0],
    ..screenwide_annotations::reveal::AnnotationReveal::WHOLE
  };
  let plain = composed(lit_middle(), vec![]);
  let drawn = composed(lit_middle(), vec![resting]);
  let most = plain
    .iter()
    .zip(&drawn)
    .map(|(a, b)| a.abs_diff(*b))
    .max()
    .unwrap();
  assert!(most <= 2, "{most}");
}

#[test]
fn a_settling_lens_smears_what_it_shows_as_it_grows() {
  // While the shutter was open the lens grew from half way: (238, 100) was
  // off the smaller lens, dark, and is lit once it has grown and enlarged.
  let mut settling = lens(2.0);
  settling.reveal = screenwide_annotations::reveal::AnnotationReveal {
    previous: [0.0, 1.0, 0.5, 1.0],
    ..screenwide_annotations::reveal::AnnotationReveal::WHOLE
  };
  let smeared = red(&composed(lit_middle(), vec![settling]), 238, 100);
  assert!(
    smeared > 40 && smeared < 200,
    "part-lit as it grew, not {smeared}"
  );
  let still = red(&composed(lit_middle(), vec![lens(2.0)]), 238, 100);
  assert!(still > 240, "a settled lens shows it lit, not {still}");
}

#[test]
fn a_redaction_under_the_lens_stays_hidden_in_it() {
  let secret: Vec<u8> = [255; 4].repeat(SIZE.0 as usize * SIZE.1 as usize);
  let mut cover = new_redact("redact".to_owned(), point(190.0, 70.0), None);
  cover.shape = AnnotationShape::Redact {
    start: point(190.0, 70.0),
    end: point(250.0, 130.0),
    seed: 1,
  };
  cover.style.redaction = AnnotationRedaction::Color;
  cover.style.color = "#000000".to_owned();
  let drawn = composed(secret, vec![cover, lens(2.0)]);
  assert!(red(&drawn, 220, 100) < 10);
}

#[test]
fn a_lens_shows_nothing_the_crop_cut_away() {
  // Dark on the left, bright on the right, and the crop keeps only the left,
  // laid on a black canvas.
  let source: Vec<u8> = (0..SIZE.1)
    .flat_map(|_| (0..SIZE.0).flat_map(|x| if x < 160 { [20, 20, 20, 255] } else { [255; 4] }))
    .collect();
  let source = screenwide_imaging::CapturedImage {
    rgba: source,
    width: SIZE.0,
    height: SIZE.1,
  };
  let mut settings = screenwide_output::fixtures::test_output_settings(SIZE.0, SIZE.1);
  settings.background_color = "#000000".to_owned();
  settings.background_type = "solid".to_owned();
  settings.crop_height = f64::from(SIZE.1);
  settings.crop_width = 160.0;
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = f64::from(SIZE.0);
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  // The lens straddles the crop's edge: its right half would show what lies
  // past it.
  settings.annotations = vec![lens_at(point(150.0, 100.0), 2.0)];
  let drawn = screenwide_compositor::compositor::still_overlay::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap();
  let at = |x: u32, y: u32| drawn.rgba[((y * drawn.width + x) * 4) as usize];
  assert!(at(180, 100) < 40, "{}", at(180, 100));
  assert!(at(130, 100) < 40, "{}", at(130, 100));
}

/// A blue arrow along y = `y` from x = `from` to `to`.
fn blue_arrow(from: f64, to: f64, y: f64) -> Annotation {
  let mut arrow = screenwide_annotations::arrow::model::new_arrow(
    "arrow".to_owned(),
    point(from, y),
    point(to, y),
    None,
  );
  arrow.style.color = "#0000ff".to_owned();
  arrow
}

fn spotlight(blur: bool) -> Annotation {
  let mut spotlight = screenwide_annotations::spotlight::model::new_spotlight(
    "spotlight".to_owned(),
    [point(0.0, 0.0), point(40.0, 40.0)],
    None,
  );
  spotlight.style.blur = blur;
  spotlight
}

#[test]
fn a_lens_enlarges_the_marks_below_it_and_not_those_above() {
  // The arrow runs through the lens's lit middle along y = 100, 8 wide: the
  // lens shows it twice as wide, so (220, 105) lies on it only below.
  let arrow = || blue_arrow(180.0, 300.0, 100.0);
  let below = composed(lit_middle(), vec![arrow(), lens(2.0)]);
  assert!(
    red(&below, 220, 105) < 40 && blue(&below, 220, 105) > 200,
    "{} {}",
    red(&below, 220, 105),
    blue(&below, 220, 105)
  );
  let above = composed(lit_middle(), vec![lens(2.0), arrow()]);
  assert!(red(&above, 220, 105) > 240, "{}", red(&above, 220, 105));
  // Over the lens, the arrow is drawn as it is.
  assert!(blue(&above, 220, 100) > 200);
}

#[test]
fn a_spotlight_shades_the_marks_below_it_and_not_those_above() {
  let blue = |rgba: &[u8]| rgba[((170 * SIZE.0 + 240) * 4 + 2) as usize];
  let arrow = || blue_arrow(180.0, 300.0, 170.0);
  let below = composed(lit_middle(), vec![arrow(), spotlight(false)]);
  let above = composed(lit_middle(), vec![spotlight(false), arrow()]);
  assert!(blue(&above) > 240, "{}", blue(&above));
  assert!(blue(&below) < 200, "{}", blue(&below));
}

#[test]
fn a_blurring_spotlight_softens_the_marks_below_it_and_not_those_above() {
  // Across the arrow's edge, over a picture that is flat there, so only the
  // arrow's own blur can tell a blurring spotlight from one that only shades.
  let across = |rgba: &[u8]| -> Vec<i32> {
    (155..186)
      .map(|y| i32::from(rgba[((y * SIZE.0 + 240) * 4 + 2) as usize]))
      .collect()
  };
  let arrow = || blue_arrow(180.0, 300.0, 170.0);
  let difference = |blur_layers: Vec<Annotation>, shade_layers: Vec<Annotation>| {
    across(&composed(lit_middle(), blur_layers))
      .iter()
      .zip(across(&composed(lit_middle(), shade_layers)))
      .map(|(blurred, shaded)| (blurred - shaded).abs())
      .max()
      .unwrap()
  };
  let below = difference(
    vec![arrow(), spotlight(true)],
    vec![arrow(), spotlight(false)],
  );
  assert!(below > 15, "the arrow below is softened, not {below}");
  let above = difference(
    vec![spotlight(true), arrow()],
    vec![spotlight(false), arrow()],
  );
  assert!(above <= 2, "the arrow above stays sharp, not {above}");
}

#[test]
fn a_blurred_mark_fades_smoothly_rather_than_in_copies() {
  // A canvas large enough that the blur spans several pixels: its deviation
  // is the shorter side over 250, here 4.8.
  const LARGE: (u32, u32) = (2000, 1200);
  let flat = [20, 20, 20, 255].repeat(LARGE.0 as usize * LARGE.1 as usize);
  let mut arrow = blue_arrow(600.0, 1800.0, 800.0);
  arrow.style.width = 40.0;
  let mut light = spotlight(true);
  light.shape = AnnotationShape::Spotlight {
    start: point(0.0, 0.0),
    end: point(200.0, 200.0),
  };
  let drawn = composed_at(LARGE, flat, vec![arrow, light]);
  // Down from the arrow's middle, across its lower edge.
  let blue: Vec<f32> = (800..860)
    .map(|y| f32::from(drawn[((y * LARGE.0 + 1200) * 4 + 2) as usize]))
    .collect();
  let falls: Vec<f32> = blue.windows(2).map(|pair| pair[0] - pair[1]).collect();
  assert!(
    falls.iter().all(|&fall| fall >= -1.0),
    "it never rises again: {blue:?}"
  );
  // A Gaussian edge falls fastest at one place, its fall growing to there and
  // shrinking after, give or take the rounding to eight bits; copies of a
  // sharp edge fall in steps, their fall swinging up and down.
  let steepest = (0..falls.len())
    .max_by(|&a, &b| falls[a].total_cmp(&falls[b]))
    .unwrap();
  let rising = falls[..=steepest]
    .windows(2)
    .all(|pair| pair[1] >= pair[0] - 1.5);
  let easing = falls[steepest..]
    .windows(2)
    .all(|pair| pair[1] <= pair[0] + 1.5);
  assert!(rising && easing, "one fall, not steps: {blue:?}");
  let widest = falls.iter().cloned().fold(0.0, f32::max);
  assert!(widest < 60.0, "softened over several pixels: {blue:?}");
}
