// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_annotations::arrow::new_arrow;
use screenwide_annotations::counter::new_counter;
use screenwide_annotations::{AnnotationHead, AnnotationPoint, AnnotationStyle};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 120;

fn style(width: f64) -> AnnotationStyle {
  AnnotationStyle {
    align: Default::default(),
    blur: false,
    color: "#ff0000".to_owned(),
    head: AnnotationHead::End,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: 0.0,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: 0.0,
    width,
  }
}

/// `annotations` drawn by the overlay's renderer over a cleared target, as
/// premultiplied BGRA rows.
fn drawn(annotations: &[screenwide_annotations::Annotation]) -> Vec<u8> {
  drawn_over(annotations, None)
}

/// `annotations` drawn over `desktop`, the underlay a highlight recolours
/// and a focus blurs.
fn drawn_over(
  annotations: &[screenwide_annotations::Annotation],
  desktop: Option<&screenwide_imaging::CapturedImage>,
) -> Vec<u8> {
  let renderer = Renderer::new().expect("the overlay renderer");
  let gpu = renderer.gpu();
  let underlay = desktop.map(|image| underlay::upload(gpu, image).expect("the underlay uploads"));
  let focus = desktop.map(|image| {
    renderer
      .chains()
      .chain(gpu, image)
      .expect("the focus chain is made")
  });
  let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: WIDTH,
      height: HEIGHT,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: screenwide_gpu::surface::FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  renderer
    .draw_arrows(
      &target.create_view(&Default::default()),
      (WIDTH, HEIGHT),
      &placed_arrows(annotations, (0.0, 0.0), (1.0, 1.0), 1.0, None, None),
      underlay.as_ref(),
      focus.as_ref(),
    )
    .expect("the annotations draw");
  gpu.read_texture(&target).expect("the frame reads back")
}

fn pixel(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
  let start = ((y * WIDTH + x) * 4) as usize;
  [
    pixels[start],
    pixels[start + 1],
    pixels[start + 2],
    pixels[start + 3],
  ]
}

#[test]
fn an_arrow_is_drawn_in_its_colour_and_nothing_else_is_covered() {
  let arrow = new_arrow(
    "arrow".to_owned(),
    AnnotationPoint { x: 20.0, y: 20.0 },
    AnnotationPoint { x: 100.0, y: 100.0 },
    Some(&style(8.0)),
  );
  let pixels = drawn(&[arrow]);
  // Half way along the shaft: opaque red, as BGRA.
  assert_eq!(pixel(&pixels, 60, 60), [0, 0, 255, 255]);
  // Past the tip and well off the shaft: the desktop shows through.
  assert_eq!(pixel(&pixels, 150, 110), [0, 0, 0, 0]);
  assert_eq!(pixel(&pixels, 10, 110), [0, 0, 0, 0]);
}

/// A counter's number is type set by the platform's text engine into the
/// atlas, then sampled by the shader: the disc carries its colour and the
/// number reads on it in white.
#[test]
fn a_counter_shows_its_number_on_its_disc() {
  let center = (80, 60);
  let mut dress = style(48.0);
  dress.head = AnnotationHead::None;
  let counter = new_counter(
    "counter".to_owned(),
    AnnotationPoint {
      x: f64::from(center.0),
      y: f64::from(center.1),
    },
    8,
    Some(&dress),
    None,
  );
  let pixels = drawn(&[counter]);
  // Inside the disc, west of the number: the disc's own red.
  assert_eq!(pixel(&pixels, center.0 - 19, center.1), [0, 0, 255, 255]);
  assert_eq!(pixel(&pixels, 5, 5), [0, 0, 0, 0]);
  let number = (center.1 - 10..center.1 + 10)
    .flat_map(|y| (center.0 - 8..center.0 + 8).map(move |x| (x, y)))
    .filter(|&(x, y)| {
      let [blue, green, red, alpha] = pixel(&pixels, x, y);
      alpha == 255 && blue > 200 && green > 200 && red > 200
    })
    .count();
  assert!(number > 10, "only {number} white pixels of the number");
}

/// A focus drawn live leaves its box to the live desktop and lays the
/// desktop it was drawn on, softened, over everything well past it.
#[test]
fn a_focus_blurs_around_its_box_and_leaves_the_box_live() {
  // A one-pixel checkerboard, which any blur turns grey.
  let rgba = (0..HEIGHT)
    .flat_map(|y| (0..WIDTH).map(move |x| if (x + y) % 2 == 0 { 255 } else { 0 }))
    .flat_map(|value| [value, value, value, 255])
    .collect();
  let desktop = screenwide_imaging::CapturedImage {
    width: WIDTH,
    height: HEIGHT,
    rgba,
  };
  let focus = screenwide_annotations::focus::model::new_focus(
    "focus".to_owned(),
    [
      AnnotationPoint { x: 60.0, y: 40.0 },
      AnnotationPoint { x: 100.0, y: 80.0 },
    ],
    None,
  );
  let pixels = drawn_over(&[focus], Some(&desktop));
  // Inside the box: nothing laid, so the live desktop shows.
  assert_eq!(pixel(&pixels, 80, 60)[3], 0);
  // Well past it: the desktop it was drawn on, opaque and softened to grey.
  let [blue, _, _, alpha] = pixel(&pixels, 5, 5);
  assert_eq!(alpha, 255);
  assert!((60..200).contains(&blue), "{blue} is not softened");
}
