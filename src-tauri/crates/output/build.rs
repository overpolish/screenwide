// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  screenwide_wgsl::assemble(&[(
    "mesh",
    &[
      "src/mesh_generator_common.wgsl",
      "src/mesh_generators.wgsl",
      "src/mesh_generators_layered.wgsl",
      "src/mesh.wgsl",
    ],
  )]);
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Background pictures are decoded with ImageIO, which reads what the
    // `image` crate cannot, such as HEIC.
    println!("cargo:rerun-if-changed=src/macos/image_decode_macos.m");
    cc::Build::new()
      .file("src/macos/image_decode_macos.m")
      .flag("-fobjc-arc")
      .compile("screenwide_output_image_decode");
    for framework in ["CoreGraphics", "Foundation", "ImageIO"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
