// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction pre-pass on real D3D11 hardware: what a secure box shows
//! is the same whatever it covers, a classic block is the exact average of
//! what it covers, and a rounded box covers every pixel its outline touches.

use super::render_test_helpers::{device, read_pixel, read_top_strip, target, OUTPUT};
use super::*;
use crate::editor::annotations::redact::new_redact;
use crate::editor::annotations::redact::records::{
  RedactRecord, RedactRecords, REDACT_FLAT, REDACT_MOSAIC,
};
use crate::editor::annotations::{
  Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape,
};
use crate::editor::preview_platform::ComposedFrame;
use crate::screenshots::{test_output_settings, CapturedImage};

/// A 64 pixel square screenshot, white round a 32 pixel square of black and
/// white drawn by `secret`, which says whether a pixel inside is black.
fn picture(secret: impl Fn(u32, u32) -> bool) -> CapturedImage {
  let mut rgba = Vec::with_capacity(64 * 64 * 4);
  for y in 0..64 {
    for x in 0..64 {
      let inside = (16..48).contains(&x) && (16..48).contains(&y);
      let value = if inside && secret(x, y) { 0 } else { 255 };
      rgba.extend_from_slice(&[value, value, value, 255]);
    }
  }
  CapturedImage {
    width: 64,
    height: 64,
    rgba,
  }
}

/// A box over exactly the secret, redacted the way `redaction` says.
fn redaction(redaction: AnnotationRedaction) -> Annotation {
  let point = |x, y| AnnotationPoint { x, y };
  let mut annotation = new_redact("secret".to_owned(), point(16.0, 16.0), None);
  if let AnnotationShape::Redact { end, .. } = &mut annotation.shape {
    *end = point(48.0, 48.0);
  }
  annotation.style.redaction = redaction;
  annotation.style.color = "#3366CC".to_owned();
  annotation
}

/// The canvas's top rows with `image` drawn at one canvas pixel a source
/// pixel, under `annotations`.
fn composed(
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  compositor: &Compositor,
  image: &CapturedImage,
  annotations: Vec<Annotation>,
) -> Vec<u8> {
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  settings.drop_shadow = false;
  (
    settings.image_width,
    settings.crop_width,
    settings.crop_height,
  ) = (64.0, 64.0, 64.0);
  (
    settings.image_x,
    settings.image_y,
    settings.crop_x,
    settings.crop_y,
  ) = (400.0, 0.0, 400.0, 0.0);
  settings.annotations = annotations;
  let source = compositor.screenshot_source(device, image).unwrap();
  let prepared = super::super::annotation::prepared_arrows(
    &settings.annotations,
    (image.width, image.height),
    &settings,
    Some(image),
    None,
    None,
  )
  .unwrap();
  let output = target(device);
  compositor
    .draw_with_camera(
      context,
      &output,
      &source,
      &settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
    )
    .unwrap();
  read_top_strip(device, context, &output)
}

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn a_secure_box_shows_the_same_whatever_it_covers() {
  let (device, context) = device();
  let compositor = Compositor::new(&device).unwrap();
  // The same two inks in the same shares, laid out differently: all a
  // secure pixelation may read of what it covers is which inks are there.
  let checks = picture(|x, y| (x + y) % 2 == 0);
  let stripes = picture(|x, _| x % 2 == 0);
  for mode in [
    AnnotationRedaction::Erase,
    AnnotationRedaction::Color,
    AnnotationRedaction::Pixelate,
  ] {
    let over_checks = composed(
      &device,
      &context,
      &compositor,
      &checks,
      vec![redaction(mode)],
    );
    let over_stripes = composed(
      &device,
      &context,
      &compositor,
      &stripes,
      vec![redaction(mode)],
    );
    assert_eq!(over_checks, over_stripes, "{mode:?} shows what it covers");
    let bare = composed(&device, &context, &compositor, &checks, Vec::new());
    assert_ne!(over_checks, bare, "{mode:?} covered nothing");
  }
}

/// A 32 pixel square BGRA source, `colour` giving each pixel's RGBA.
fn source(
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  compositor: &Compositor,
  colour: impl Fn(u32, u32) -> [u8; 4],
) -> SourceTexture {
  let source = compositor.source(device, (32, 32)).unwrap();
  let bytes: Vec<u8> = (0..32)
    .flat_map(|y| (0..32).map(move |x| (x, y)))
    .flat_map(|(x, y)| {
      let [red, green, blue, alpha] = colour(x, y);
      [blue, green, red, alpha]
    })
    .collect();
  let resource: ID3D11Resource = source.texture.cast().unwrap();
  unsafe { context.UpdateSubresource(&resource, 0, None, bytes.as_ptr().cast(), 32 * 4, 0) };
  source
}

/// `record` applied to `source`, and the copy it left.
fn redacted(
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  compositor: &Compositor,
  source: &SourceTexture,
  record: RedactRecord,
) -> ID3D11Texture2D {
  let records = RedactRecords {
    records: vec![record],
    zones: Vec::new(),
  };
  let view = compositor
    .redactor
    .apply(device, context, source, &records, false)
    .unwrap()
    .expect("a box covering the source redacts it");
  unsafe { view.GetResource() }.unwrap().cast().unwrap()
}

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn a_classic_block_is_the_exact_average_of_what_it_covers() {
  let (device, context) = device();
  let compositor = Compositor::new(&device).unwrap();
  let half = source(&device, &context, &compositor, |x, _| {
    if x < 16 {
      [0, 0, 0, 255]
    } else {
      [255, 255, 255, 255]
    }
  });
  let record = RedactRecord {
    bounds: [0, 0, 32, 32],
    source_width: 32,
    mode: REDACT_MOSAIC,
    size: 32.0,
    color: [0.0, 0.0, 0.0, 1.0],
    grid: [1, 1],
    ..RedactRecord::default()
  };
  let copy = redacted(&device, &context, &compositor, &half, record);
  // 127.5 rounds half up, as the Metal pass rounds it.
  for (x, y) in [(2, 2), (30, 30)] {
    assert_eq!(
      read_pixel(&device, &context, &copy, x, y),
      [128, 128, 128, 255]
    );
  }
}

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn a_rounded_box_covers_every_pixel_its_outline_touches() {
  let (device, context) = device();
  let compositor = Compositor::new(&device).unwrap();
  let red = source(&device, &context, &compositor, |_, _| [255, 0, 0, 255]);
  let record = RedactRecord {
    bounds: [0, 0, 32, 32],
    source_width: 32,
    mode: REDACT_FLAT,
    color: [0.0, 0.0, 1.0, 1.0],
    radius: 16.0,
    ..RedactRecord::default()
  };
  let copy = redacted(&device, &context, &compositor, &red, record);
  let blue = [255, 0, 0, 255];
  // Wholly outside the circle: the picture the box never covered.
  assert_eq!(read_pixel(&device, &context, &copy, 0, 0), [0, 0, 255, 255]);
  // Touched by the outline at the middle of an edge, and the centre.
  assert_eq!(read_pixel(&device, &context, &copy, 0, 16), blue);
  assert_eq!(read_pixel(&device, &context, &copy, 16, 16), blue);
}
