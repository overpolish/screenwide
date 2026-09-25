// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{device, read_top_strip, target, OUTPUT};
use super::*;
use crate::editor::preview_platform::surface::selection::SelectionOverlay;
use crate::editor::preview_platform::ComposedFrame;
use crate::screenshots::test_output_settings;
use windows::core::Interface;
use windows::Win32::Graphics::DirectComposition::{DCompositionCreateDevice, IDCompositionDevice};
use windows::Win32::Graphics::Dxgi::{IDXGIDevice, IDXGIFactory2};

#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn compositor_background_survives_selection_overlay_state() {
  let (device, context) = device();
  let dxgi: IDXGIDevice = device.cast().unwrap();
  let adapter = unsafe { dxgi.GetAdapter() }.unwrap();
  let factory: IDXGIFactory2 = unsafe { adapter.GetParent() }.unwrap();
  let composition: IDCompositionDevice = unsafe { DCompositionCreateDevice(&dxgi) }.unwrap();
  let root = unsafe { composition.CreateVisual() }.unwrap();
  let mut selection = SelectionOverlay::new(&device, &factory, &composition, &root).unwrap();
  selection
    .draw(
      &device,
      &context,
      OUTPUT,
      Some([10.0, 10.0, 100.0, 100.0]),
      None,
      None,
      0.0,
      None,
      None,
      None,
      None,
      None,
      &[],
      1.0,
      false,
    )
    .unwrap();
  let compositor = Compositor::new(&device).unwrap();
  let source = compositor
    .screenshot_source(
      &device,
      &crate::screenshots::CapturedImage {
        width: 64,
        height: 64,
        rgba: vec![255; 64 * 64 * 4],
      },
    )
    .unwrap();
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_seed = 34_644;
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.mesh_points.clear();
  let draw = |target: &ID3D11Texture2D| {
    compositor
      .draw_with_camera(
        &context,
        target,
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
        &Default::default(),
      )
      .unwrap();
  };
  let first = target(&device);
  draw(&first);
  let first_pixels = read_top_strip(&device, &context, &first);
  selection
    .draw(
      &device,
      &context,
      OUTPUT,
      None,
      None,
      None,
      0.0,
      None,
      None,
      None,
      None,
      None,
      &[],
      1.0,
      false,
    )
    .unwrap();
  let second = target(&device);
  draw(&second);
  assert_eq!(first_pixels, read_top_strip(&device, &context, &second));
}
