// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "src/shader_layouts.rs"]
mod shader_layouts;

use screenwide_wgsl::{assembled, Precompiled};

fn main() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  screenwide_wgsl::assemble(&[
    ("camera_thumbnail", &["src/shaders/camera_thumbnail.wgsl"]),
    (
      "desktop_compositor",
      &["src/shaders/desktop_compositor.wgsl"],
    ),
  ]);
  let mut shaders = vec![Precompiled {
    name: "DESKTOP_COMPOSITOR",
    source: assembled("desktop_compositor"),
    layout: &shader_layouts::DESKTOP_COMPOSITOR,
    entry_points: &["vs_main", "fs_main"],
  }];
  // Only the macOS camera scales its thumbnails on the GPU.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    shaders.push(Precompiled {
      name: "CAMERA_THUMBNAIL",
      source: assembled("camera_thumbnail"),
      layout: &shader_layouts::CAMERA_THUMBNAIL,
      entry_points: &["vs_main", "fs_bgra", "fs_biplanar"],
    });
  }
  screenwide_wgsl::precompile("precompiled_shaders.rs", &shaders);
}
