// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::desktop_capture::PixelRect;

#[test]
fn constants_keep_monitor_crop_and_canvas_placement_separate() {
  let constants = PieceConstants::new(
    [1600, 900],
    [3840, 2160],
    CapturePiece {
      display_id: 7,
      source_pixels: PixelRect {
        x: 120,
        y: 80,
        width: 1920,
        height: 1080,
      },
      destination: PixelRect {
        x: 400,
        y: 0,
        width: 1200,
        height: 900,
      },
    },
  );
  assert_eq!(constants.output_size, [1600, 900]);
  assert_eq!(constants.source_size, [3840, 2160]);
  assert_eq!(constants.source_origin, [120, 80]);
  assert_eq!(constants.destination_origin, [400, 0]);
}

/// The canvas the build's shader draws matches the one the same shader
/// compiled from WGSL draws, two pieces side by side: each piece's place is
/// read by the vertex stage at its own dynamic offset, so a slot laid out
/// wrongly puts both pieces in one place or none.
#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn the_precompiled_canvas_draws_what_wgsl_draws() {
  let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Light).expect("the shared GPU");
  let (width, height) = (64, 32);
  let picture = |colour: [u8; 4]| {
    gpu.texture_with_pixels(
      "Screenwide test piece",
      (8, 8, 1),
      FORMAT,
      &colour.repeat(64),
    )
  };
  let (left, right) = (picture([0, 0, 255, 255]), picture([0, 255, 0, 255]));
  let piece = |x| CapturePiece {
    display_id: 1,
    source_pixels: PixelRect {
      x: 0,
      y: 0,
      width: 8,
      height: 8,
    },
    destination: PixelRect {
      x,
      y: 0,
      width: 32,
      height: 32,
    },
  };
  let drawn = |canvas: &DesktopCanvas| {
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide test canvas"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let sources = [
      PieceSource {
        piece: piece(0),
        texture: &left,
      },
      PieceSource {
        piece: piece(32),
        texture: &right,
      },
    ];
    canvas
      .draw(&target.create_view(&Default::default()), &sources)
      .expect("the canvas draws");
    gpu.read_texture(&target).expect("the canvas reads back")
  };
  let precompiled = drawn(&DesktopCanvas::new(&gpu, width, height, 2));
  let pixel = |pixels: &[u8], x: usize| pixels[x * 4..x * 4 + 4].to_vec();
  assert_eq!(pixel(&precompiled, 4), [0, 0, 255, 255], "the left piece");
  assert_eq!(pixel(&precompiled, 60), [0, 255, 0, 255], "the right piece");
  let compiled = screenwide_gpu::compiling_wgsl(|| DesktopCanvas::new(&gpu, width, height, 2));
  assert!(
    precompiled == drawn(&compiled),
    "the precompiled canvas draws different pixels"
  );
}
