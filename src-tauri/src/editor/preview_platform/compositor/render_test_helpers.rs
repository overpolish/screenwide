// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) const OUTPUT: (u32, u32) = (1920, 1080);

pub(super) fn gpu() -> &'static Gpu {
  crate::gpu::shared().expect("the shared GPU")
}

pub(super) fn compositor() -> Compositor {
  // One transparent texel per cursor style: these tests draw no cursor.
  let cursors = NativeCursors {
    size: (1, 1),
    layers: vec![0; 4 * 8],
    hotspots: [[0.0; 4]; 8],
  };
  Compositor::new(gpu(), cursors).expect("the preview compositor")
}

pub(super) fn target() -> wgpu::Texture {
  target_size(OUTPUT)
}

pub(super) fn target_size(size: (u32, u32)) -> wgpu::Texture {
  gpu().device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide test target"),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  })
}

pub(super) fn view(target: &wgpu::Texture) -> wgpu::TextureView {
  target.create_view(&Default::default())
}

/// `source` composed into `target` with `settings`, nothing over it.
pub(super) fn draw(
  compositor: &Compositor,
  target: &wgpu::Texture,
  source: &SourceTexture,
  settings: &ScreenshotOutputSettings,
) {
  compositor
    .draw_with_camera(
      &view(target),
      source,
      settings,
      crate::editor::preview_platform::ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &Default::default(),
    )
    .expect("the canvas draws");
}

/// A white screenshot source of `size`.
pub(super) fn white_source(compositor: &Compositor, size: (u32, u32)) -> SourceTexture {
  compositor
    .screenshot_source(&crate::screenshots::CapturedImage {
      width: size.0,
      height: size.1,
      rgba: vec![255; size.0 as usize * size.1 as usize * 4],
    })
    .expect("the source uploads")
}
/// The BGRA pixel at `x`, `y`.
pub(super) fn read_pixel(target: &wgpu::Texture, x: u32, y: u32) -> [u8; 4] {
  let pixels = gpu().read_texture(target).expect("the target reads back");
  let at = (y * target.width() + x) as usize * 4;
  [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

/// The top 64 rows, BGRA.
pub(super) fn read_top_strip(target: &wgpu::Texture) -> Vec<u8> {
  let mut pixels = gpu().read_texture(target).expect("the target reads back");
  pixels.truncate(target.width() as usize * 64 * 4);
  pixels
}
