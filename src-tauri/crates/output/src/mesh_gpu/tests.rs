// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Instant;

use super::*;
use crate::mesh_generator::{every_generator, mesh_generator};

fn classic() -> &'static MeshGenerator {
  mesh_generator("mesh").expect("the app's own mesh is a generator")
}

#[test]
#[ignore = "4K GPU stress test"]
fn renders_and_encodes_a_4k_mesh_without_noise_bloat() {
  let colors = [
    [255, 46, 129, 255],
    [34, 211, 238, 255],
    [250, 204, 21, 255],
    [99, 102, 241, 255],
    [17, 24, 39, 255],
  ];
  let points = [
    MeshGradientPoint {
      radius_x: 80.0,
      radius_y: 52.0,
      rotation: 24.0,
      x: 12.0,
      y: 18.0,
    },
    MeshGradientPoint {
      radius_x: 60.0,
      radius_y: 88.0,
      rotation: -38.0,
      x: 88.0,
      y: 14.0,
    },
    MeshGradientPoint {
      radius_x: 94.0,
      radius_y: 48.0,
      rotation: 72.0,
      x: 22.0,
      y: 90.0,
    },
    MeshGradientPoint {
      radius_x: 54.0,
      radius_y: 82.0,
      rotation: -12.0,
      x: 92.0,
      y: 84.0,
    },
  ];
  let started = Instant::now();
  let image = render(3840, 2160, classic(), &colors, &points, 42, 10.0, 0.0).unwrap();
  let rendered_in = started.elapsed();
  let (width, height) = image.dimensions();
  let mut image = screenwide_imaging::CapturedImage {
    height,
    rgba: image.into_raw(),
    width,
  };
  image.rgba[3] = 0;
  let encoding_started = Instant::now();
  let encoded = crate::encoding::encode_png(&image).unwrap();
  eprintln!(
    "4K mesh: GPU render/readback {rendered_in:?}, PNG encode {:?}, {} bytes",
    encoding_started.elapsed(),
    encoded.len()
  );
  assert_eq!((image.width, image.height), (3840, 2160));
  assert!(
    encoded.len() < 3_000_000,
    "anti-banding noise made the empty mesh PNG {} bytes",
    encoded.len()
  );
}

#[test]
fn mesh_contains_distinct_colour_regions() {
  let colors = [
    [255, 20, 40, 255],
    [20, 255, 60, 255],
    [30, 60, 255, 255],
    [10, 10, 10, 255],
  ];
  let points = [
    MeshGradientPoint {
      radius_x: 48.0,
      radius_y: 42.0,
      rotation: 0.0,
      x: 8.0,
      y: 12.0,
    },
    MeshGradientPoint {
      radius_x: 46.0,
      radius_y: 52.0,
      rotation: 24.0,
      x: 92.0,
      y: 18.0,
    },
    MeshGradientPoint {
      radius_x: 54.0,
      radius_y: 44.0,
      rotation: -31.0,
      x: 48.0,
      y: 94.0,
    },
  ];
  let image = render(640, 360, classic(), &colors, &points, 42, 7.0, 0.0).unwrap();
  let samples = [
    image.get_pixel(48, 40).0,
    image.get_pixel(590, 54).0,
    image.get_pixel(320, 330).0,
  ];
  let channel_range = |channel: usize| {
    let minimum = samples.iter().map(|pixel| pixel[channel]).min().unwrap();
    let maximum = samples.iter().map(|pixel| pixel[channel]).max().unwrap();
    maximum - minimum
  };
  assert!(
    channel_range(0) > 80 && channel_range(1) > 80 && channel_range(2) > 80,
    "mesh samples were unexpectedly flat: {samples:?}"
  );
}

/// Every ported generator paints something, paints more than one colour, and
/// paints something else five seconds later.
///
/// A generator that compiled but read the wrong uniforms would come back flat,
/// which is what the spread catches. A generator whose `time` never reached
/// the maths it is meant to drive would paint the same picture at every
/// moment, which is what the second render catches: the background behind a
/// recording would sit dead still while the classic mesh beside it moves.
#[test]
fn every_generator_paints_a_varied_picture() {
  let colors = [
    [12, 18, 52, 255],
    [40, 130, 200, 255],
    [220, 120, 180, 255],
    [250, 230, 190, 255],
  ];
  for generator in every_generator().iter().skip(1) {
    let image = render(32, 32, generator, &colors, &[], 1_234, 0.0, 0.0)
      .unwrap_or_else(|error| panic!("{} did not render: {error}", generator.name));
    assert_eq!(image.dimensions(), (32, 32));
    let channel = |index: usize| {
      let values = image.pixels().map(|pixel| pixel.0[index]);
      let minimum = values.clone().min().unwrap();
      let maximum = values.max().unwrap();
      maximum - minimum
    };
    assert!(
      channel(0) > 4 || channel(1) > 4 || channel(2) > 4,
      "{} painted a flat swatch",
      generator.name
    );
    // Randomize changes the seed and nothing else, so a seed that painted
    // the same picture would leave the button doing nothing.
    let reseeded = render(32, 32, generator, &colors, &[], 98_765, 0.0, 0.0).unwrap();
    assert_ne!(
      image.as_raw(),
      reseeded.as_raw(),
      "{} ignored its seed",
      generator.name
    );
    // Five seconds in, which is inside a short clip, the picture has moved.
    let later = render(32, 32, generator, &colors, &[], 1_234, 0.0, 5.0).unwrap();
    assert_ne!(
      image.as_raw(),
      later.as_raw(),
      "{} did not drift over five seconds",
      generator.name
    );
  }
}

/// The compute stage the build compiled paints what the same shader compiled
/// from WGSL paints: a binding laid out wrongly writes nothing, or reads the
/// wrong parameters.
#[test]
fn the_precompiled_mesh_paints_what_wgsl_paints() {
  let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Light).expect("the shared GPU");
  let precompiled = Renderer::new(&gpu).expect("the mesh renderer");
  let compiled = screenwide_gpu::compiling_wgsl(|| Renderer::new(&gpu)).expect("the mesh renderer");
  let generator = &every_generator()[1];
  let mut uniforms = MeshUniforms {
    dimensions: [48, 32],
    point_count: 0,
    seed: 1_234,
    warp_percent: 0.0,
    generator: generator.id,
    generator_color_count: generator.color_count as u32,
    time: 2.0,
    generator_speed: generator.speed,
    padding: [0.0; 3],
    base_color: color(&[12, 18, 52, 255]),
    points: [[0.0; 8]; MAX_POINTS],
    colors: [[0.0; 4]; MAX_POINTS],
  };
  for (index, value) in [
    [40, 130, 200, 255],
    [220, 120, 180, 255],
    [250, 230, 190, 255],
  ]
  .iter()
  .enumerate()
  {
    uniforms.colors[index] = color(value);
  }
  let painted = precompiled
    .render(48, 32, &uniforms)
    .expect("the mesh paints");
  assert!(
    painted
      .as_chunks::<4>()
      .0
      .iter()
      .any(|pixel| pixel != &[0, 0, 0, 0]),
    "the precompiled mesh painted nothing"
  );
  assert!(
    painted == compiled.render(48, 32, &uniforms).expect("the mesh paints"),
    "the precompiled mesh paints different pixels"
  );
}
