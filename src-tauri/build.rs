// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "build/wgsl.rs"]
mod wgsl;
#[path = "build/windows_manifest.rs"]
mod windows_manifest;

fn main() {
  wgsl::assemble();
  // Build scripts are compiled for the host, so `cfg!(target_os)` here answers
  // "what am I running on", not "what am I building for". Cross-compiling from
  // macOS to Windows must not hand the Objective-C sources to the MSVC target.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Headers the annotations and text crates own, included by the sources
    // below.
    println!("cargo:rerun-if-changed=crates/text/src/engine/macos/annotation_text_box.h");
    println!("cargo:rerun-if-changed=crates/annotations/src/geometry.h");
    println!("cargo:rerun-if-changed=crates/annotations/src/reveal.h");
    println!("cargo:rerun-if-changed=src/osc/cursor_macos/cursor_session_macos.m");
    println!("cargo:rerun-if-changed=src/app_windows/color_panel_macos.m");
    println!("cargo:rerun-if-changed=src/app_windows/dismissal_macos.m");
    // Headers the osc crate owns, included by the sources below.
    println!("cargo:rerun-if-changed=crates/osc/src/gpu/macos");
    println!("cargo:rerun-if-changed=src/annotate/macos/native_overlay_macos.m");
    println!("cargo:rerun-if-changed=src/annotate/macos/native_overlay_macos_input.m");
    println!("cargo:rerun-if-changed=src/annotate/macos/native_overlay_macos_private.h");
    println!("cargo:rerun-if-changed=src/annotate/macos/native_overlay_macos.h");
    println!("cargo:rerun-if-changed=src/ruler/cursor_guard_macos.m");
    println!("cargo:rerun-if-changed=src/glide/macos/spaces/native.m");
    println!("cargo:rerun-if-changed=src/glide/macos/drag_start.m");
    println!("cargo:rerun-if-changed=src/glide/macos/spaces/carry.m");
    println!("cargo:rerun-if-changed=src/glide/macos/spaces/drag_point.m");
    println!("cargo:rerun-if-changed=src/glide/macos/spaces/target.h");
    cc::Build::new()
      .file("src/glide/macos/drag_start.m")
      .file("src/glide/macos/spaces/native.m")
      .file("src/glide/macos/spaces/carry.m")
      .file("src/glide/macos/spaces/drag_point.m")
      .file("src/annotate/macos/native_overlay_macos.m")
      .file("src/annotate/macos/native_overlay_macos_input.m")
      .file("src/ruler/cursor_guard_macos.m")
      .file("src/osc/cursor_macos/cursor_session_macos.m")
      .file("src/app_windows/color_panel_macos.m")
      .file("src/app_windows/dismissal_macos.m")
      .flag("-fobjc-arc")
      .compile("screenwide_gpu_compositor");
    // Objective-C categories do not define a class symbol, so the linker will
    // otherwise leave their object files inside the static native archive.
    println!("cargo:rustc-link-arg=-ObjC");
    for framework in [
      "AVFoundation",
      "AppKit",
      "CoreMedia",
      "CoreText",
      "CoreVideo",
      "Foundation",
      "ImageIO",
      "QuartzCore",
      "UniformTypeIdentifiers",
      "VideoToolbox",
    ] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
    // Glide's tap recognition reads raw trackpad contact frames, which only
    // the private MultitouchSupport framework exposes.
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=MultitouchSupport");
  }
  let attributes =
    tauri_build::Attributes::new().windows_attributes(windows_manifest::windows_attributes());
  if let Err(error) = tauri_build::try_build(attributes) {
    panic!("{error:#}");
  }
}
