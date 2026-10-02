// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 16;
/// Wide enough that only the middle bar is in the viewport.
const PITCH: f32 = 100.0;
/// Four envelope points: one bucket, so the middle bar is the only one inside
/// the recording.
const POINTS: f32 = 4.0;

/// The summed alpha of one loud bar, slid left by `shift` pixels.
fn coverage(
  painter: &Painter,
  gpu: &Gpu,
  levels: &wgpu::BindGroup,
  shift: f32,
  half_height: f32,
) -> u64 {
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
  // The row slides by the playhead's progress through its bucket, in
  // pitches; `centre_bucket` is `playhead * (points - 1) / 4`.
  let playhead = shift / PITCH * 4.0 / (POINTS - 1.0);
  let constants = Constants {
    color: [1.0; 4],
    flat: [0.0; 4],
    geometry: [WIDTH as f32, HEIGHT as f32, PITCH, 4.0],
    // No hairline, so the bar's height is exactly `half_height`.
    style: [half_height, 0.0, playhead, POINTS],
  };
  painter.encode(
    gpu,
    levels,
    constants,
    &target.create_view(&Default::default()),
  );
  let pixels = gpu.read_texture(&target).expect("the ribbon reads back");
  let (texels, _) = pixels.as_chunks::<4>();
  texels.iter().map(|texel| u64::from(texel[3])).sum()
}

/// A bar's edge pixels are integrated, not feathered, so sliding it by any
/// fraction of a pixel keeps its total coverage: the row does not shimmer
/// while it scrolls.
#[test]
fn a_bar_keeps_its_coverage_at_every_subpixel_offset() {
  let gpu = crate::gpu::shared().expect("a graphics device");
  let painter = Painter::new(&gpu.device);
  let levels = painter.bindings(&gpu.device, &levels_view(gpu, &[1.0]));
  for half_height in [2.0, 6.0] {
    let sums = (0..16)
      .map(|phase| coverage(&painter, gpu, &levels, phase as f32 / 16.0, half_height) as f32)
      .collect::<Vec<_>>();
    let minimum = sums.iter().copied().fold(f32::MAX, f32::min);
    let maximum = sums.iter().copied().fold(f32::MIN, f32::max);
    let mean = (minimum + maximum) * 0.5;
    assert!(mean > 0.0, "the bar is drawn");
    assert!(
      (maximum - minimum) / mean < 0.005,
      "coverage drifts from {minimum} to {maximum} at half-height {half_height}"
    );
  }
}
