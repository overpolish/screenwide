// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "src/shader_layouts.rs"]
mod shader_layouts;

use screenwide_wgsl::{assembled, Precompiled};

fn main() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  screenwide_wgsl::assemble(&[(
    "osc",
    &[
      // First: it carries the module's directive.
      "src/gpu/shaders/osc.wgsl",
      "src/gpu/shaders/lens.wgsl",
      "src/gpu/shaders/reference.wgsl",
      "../gpu/src/shaders/disc_blur.wgsl",
    ],
  )]);
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[Precompiled {
      name: "OSC",
      source: assembled("osc"),
      layout: &shader_layouts::OSC,
      entry_points: &["vs_main", "fs_main"],
    }],
  );
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // The frame-resize cursors AppKit draws, which the overlays and the
    // editor share.
    println!("cargo:rerun-if-changed=src/gpu/macos");
    cc::Build::new()
      .file("src/gpu/macos/region_cursor_macos.m")
      .flag("-fobjc-arc")
      .compile("screenwide_osc_cursor");
    for framework in ["AppKit", "Foundation"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
