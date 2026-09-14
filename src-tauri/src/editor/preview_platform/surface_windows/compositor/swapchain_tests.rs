// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{device, read_pixel, target_size};
use super::*;
use crate::editor::preview_platform::ComposedFrame;
use crate::screenshots::test_output_settings;
use windows::core::Interface;
use windows::Win32::Graphics::Dxgi::{
  Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  IDXGIDevice, IDXGIFactory2, IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
  DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn compositor_background_survives_composition_swap_chain_rotation() {
  let (device, context) = device();
  let dxgi: IDXGIDevice = device.cast().unwrap();
  let adapter = unsafe { dxgi.GetAdapter() }.unwrap();
  let factory: IDXGIFactory2 = unsafe { adapter.GetParent() }.unwrap();
  let chain: IDXGISwapChain3 = unsafe {
    factory
      .CreateSwapChainForComposition(
        &device,
        &DXGI_SWAP_CHAIN_DESC1 {
          Width: 2048,
          Height: 1280,
          Format: DXGI_FORMAT_B8G8R8A8_UNORM,
          SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
          },
          BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
          BufferCount: 2,
          Scaling: DXGI_SCALING_STRETCH,
          SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
          AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
          ..Default::default()
        },
        None,
      )
      .unwrap()
      .cast()
      .unwrap()
  };
  let compositor = Compositor::new(&device).unwrap();
  let source = compositor
    .screenshot_source(
      &device,
      &crate::screenshots::CapturedImage {
        width: 1920,
        height: 1080,
        rgba: vec![255; 1920 * 1080 * 4],
      },
    )
    .unwrap();
  let mut settings = test_output_settings(1920, 1080);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.radius_percent = 8.42;
  settings.background_radius_percent = 15.9;
  settings.drop_shadow = true;
  settings.image_width = 1_074.86;
  settings.crop_width = 1_074.86;
  settings.crop_height = 604.61;
  settings.image_x = 352.35;
  settings.image_y = 126.32;
  settings.crop_x = settings.image_x;
  settings.crop_y = settings.image_y;
  let ordinary = target_size(&device, (2048, 1280));
  for seed in [34_644, 55_119] {
    settings.mesh_seed = seed;
    compositor
      .draw_with_camera(
        &context,
        &ordinary,
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
      )
      .unwrap();
    let expected = read_pixel(&device, &context, &ordinary, 1440, 32);
    eprintln!("composition test seed {seed} ordinary RGBA={expected:?}");
    for _ in 0..20 {
      let target: ID3D11Texture2D = unsafe { chain.GetBuffer(0) }.unwrap();
      compositor
        .draw_with_camera(
          &context,
          &target,
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
        )
        .unwrap();
      assert_eq!(expected, read_pixel(&device, &context, &target, 1440, 32));
      unsafe { chain.Present(0, Default::default()).unwrap() };
    }
  }
}
