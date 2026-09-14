// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::render_test_helpers::{device, read_pixel, target};
use super::*;
use crate::editor::preview_platform::ComposedFrame;
use crate::screenshots::test_output_settings;
#[test]
#[ignore = "requires a Windows D3D11 hardware adapter"]
fn aurora_background_is_stable_across_cpu_rounding_modes() {
  let (device, context) = device();
  let compositor = Compositor::new(&device).expect("D3D11 preview compositor");
  let source = compositor
    .screenshot_source(
      &device,
      &crate::screenshots::CapturedImage {
        width: 1920,
        height: 1080,
        rgba: vec![255; 1920 * 1080 * 4],
      },
    )
    .expect("source texture");
  let mut settings = test_output_settings(1920, 1080);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_seed = 34_644;
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.mesh_points.clear();
  settings.mesh_warp_percent = 10.0;
  settings.radius_percent = 8.0;
  settings.background_radius_percent = 16.0;
  settings.image_width = 400.0;
  settings.crop_width = 400.0;
  settings.crop_height = 225.0;
  settings.image_x = 20.0;
  settings.image_y = 400.0;
  settings.crop_x = settings.image_x;
  settings.crop_y = settings.image_y;
  let target = target(&device);
  unsafe fn get_mxcsr() -> u32 {
    let mut value = 0;
    std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value, options(nostack, preserves_flags));
    value
  }
  unsafe fn set_mxcsr(value: u32) {
    std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &value, options(nostack, preserves_flags));
  }
  let original = unsafe { get_mxcsr() };
  struct Restore(u32);
  impl Drop for Restore {
    fn drop(&mut self) {
      unsafe { set_mxcsr(self.0) };
    }
  }
  let _restore = Restore(original);
  let mut pixels = Vec::new();
  for rounding in [0, 0x2000, 0x4000, 0x6000] {
    unsafe { set_mxcsr((original & !0x6000) | rounding) };
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
    pixels.push(read_pixel(&device, &context, &target, 1500, 700));
  }
  assert!(
    pixels.windows(2).all(|pair| pair[0] == pair[1]),
    "Aurora changed with CPU rounding mode: {pixels:?}"
  );
}
