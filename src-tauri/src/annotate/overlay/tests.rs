// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::arrow::new_arrow;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::{AnnotationHead, AnnotationPoint, AnnotationStyle};

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
fn drawn(annotations: &[crate::editor::annotations::Annotation]) -> Vec<u8> {
  let renderer = Renderer::new().expect("the overlay renderer");
  let gpu = renderer.gpu();
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
    format: crate::gpu::surface::FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  renderer
    .draw_arrows(
      &target.create_view(&Default::default()),
      (WIDTH, HEIGHT),
      &placed_arrows(annotations, (0.0, 0.0), (1.0, 1.0), 1.0, None, None),
      None,
      None,
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
