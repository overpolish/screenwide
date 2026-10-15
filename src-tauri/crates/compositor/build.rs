// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "build/precompiled_shaders.rs"]
mod precompiled_shaders;

/// The compositor's shaders as the parts each joins, in order.
const MODULES: &[(&str, &[&str])] = &[
  ("annotation_blur", &["src/shaders/annotation_blur.wgsl"]),
  ("mips", &["src/shaders/mips.wgsl"]),
  ("focus_blur", &["src/shaders/focus_blur.wgsl"]),
  ("focus_copy", &["src/shaders/focus_copy.wgsl"]),
  (
    "preview",
    &[
      // First: it carries the module's directive.
      "src/shaders/preview.wgsl",
      "../output/src/mesh_generator_common.wgsl",
      "../output/src/mesh_generators.wgsl",
      "../output/src/mesh_generators_layered.wgsl",
      "src/shaders/annotations.wgsl",
      "src/shaders/annotation_counter.wgsl",
      "src/shaders/annotation_text.wgsl",
      "src/shaders/annotation_shape.wgsl",
      "src/shaders/annotation_draw.wgsl",
      "src/shaders/annotation_spotlight.wgsl",
      "src/shaders/annotation_magnify.wgsl",
      "src/shaders/annotation_highlight.wgsl",
      "src/shaders/annotation_image.wgsl",
      "src/shaders/annotation_layers.wgsl",
      "src/shaders/annotation_tiles.wgsl",
      "src/shaders/fsr1.wgsl",
    ],
  ),
  (
    "redact_cells",
    &["src/shaders/redact.wgsl", "src/shaders/redact_cells.wgsl"],
  ),
  (
    "redact_rows",
    &["src/shaders/redact.wgsl", "src/shaders/redact_rows.wgsl"],
  ),
  (
    "redact_paint",
    &["src/shaders/redact.wgsl", "src/shaders/redact_paint.wgsl"],
  ),
  ("video_planes", &["src/shaders/video_planes.wgsl"]),
];

fn main() {
  let out_dir =
    std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR"));
  screenwide_wgsl::assemble(MODULES);
  precompiled_shaders::precompile(&out_dir);
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Type and keyboard artwork are set by Core Text and AppKit.
    println!("cargo:rerun-if-changed=src/platform_macos/native");
    println!("cargo:rerun-if-changed=../annotations/src/text/macos/annotation_text_box.h");
    cc::Build::new()
      .file("src/platform_macos/native/type_device_macos.m")
      .file("src/platform_macos/native/gpu_compositor_macos_keyboard_artwork.m")
      .flag("-fobjc-arc")
      .compile("screenwide_compositor_type");
    for framework in ["AppKit", "CoreGraphics", "CoreText", "Foundation"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
