// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  screenwide_wgsl::assemble(&[(
    "camera_effects",
    &[
      // First: it carries the module's directive.
      "src/camera_effects/shaders/camera_effects.wgsl",
      "src/camera_effects/shaders/colour_correction.wgsl",
      "../compositor/src/shaders/fsr1.wgsl",
    ],
  )]);
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    println!("cargo:rerun-if-changed=src/cursor_export/macos/video_export");
    println!("cargo:rerun-if-changed=src/decode/macos");
    cc::Build::new()
      .file("src/cursor_export/macos/video_export/video_export.m")
      .file("src/cursor_export/macos/video_export/video_export_setup.m")
      .file("src/cursor_export/macos/video_export/video_export_inflight.m")
      .file("src/decode/macos/recording_preview_reader_macos.m")
      .file("src/decode/macos/recording_preview_scrubber_macos.m")
      .flag("-fobjc-arc")
      .compile("screenwide_export_native");
    // The compositor's Objective-C categories define no class symbol, so
    // without this the linker leaves them out of this crate's own tests.
    println!("cargo:rustc-link-arg=-ObjC");
    for framework in [
      "AVFoundation",
      "CoreMedia",
      "CoreVideo",
      "Foundation",
      "VideoToolbox",
    ] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
