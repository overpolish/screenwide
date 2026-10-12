// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  screenwide_wgsl::assemble(&[(
    "osc",
    &[
      // First: it carries the module's directive.
      "src/gpu/shaders/osc.wgsl",
      "src/gpu/shaders/lens.wgsl",
    ],
  )]);
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    println!("cargo:rerun-if-changed=src/gpu/macos");
    cc::Build::new()
      .file("src/gpu/macos/osc_material_surface_macos.m")
      .file("src/gpu/macos/osc_text_texture_macos.m")
      .file("src/gpu/macos/osc_gpu_macos+primitives.m")
      .file("src/gpu/macos/osc_gpu_macos+selection.m")
      .file("src/gpu/macos/osc_gpu_macos+crop.m")
      .file("src/gpu/macos/osc_gpu_macos+ruler.m")
      .file("src/gpu/macos/osc_gpu_pipeline_macos.m")
      .file("src/gpu/macos/region_cursor_macos.m")
      .file("src/gpu/macos/region_magnifier_macos.m")
      .flag("-fobjc-arc")
      .compile("screenwide_osc_gpu");
    // Objective-C categories define no class symbol, so the linker would
    // otherwise leave their objects out of this crate's own tests.
    println!("cargo:rustc-link-arg=-ObjC");
    for framework in ["AppKit", "CoreText", "Foundation", "Metal", "QuartzCore"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
