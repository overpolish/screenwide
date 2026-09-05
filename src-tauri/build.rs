// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  #[cfg(windows)]
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
    compile_windows_preview_shaders();
  }
  // Build scripts are compiled for the host, so `cfg!(target_os)` here answers
  // "what am I running on", not "what am I building for". Cross-compiling from
  // macOS to Windows must not hand the Objective-C sources to the MSVC target.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    println!("cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos.m");
    println!("cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos+presenter.m");
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos+presenter_keyboard.m"
    );
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_cursor_resources.m"
    );
    println!("cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_keyboard.m");
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_keyboard_artwork.m"
    );
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_shader_source.h"
    );
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_keyboard_shader_source.h"
    );
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_gpu_macos.h");
    println!("cargo:rerun-if-changed=src/exports/osc_controls.h");
    println!("cargo:rerun-if-changed=src/exports/osc_material_surface_macos.h");
    println!("cargo:rerun-if-changed=src/exports/osc_material_surface_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_text_texture_macos.h");
    println!("cargo:rerun-if-changed=src/exports/osc_text_texture_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_icon_renderer_macos.m");
    println!("cargo:rerun-if-changed=src/exports/region_cursor_macos.m");
    println!("cargo:rerun-if-changed=src/exports/cursor_session_macos.m");
    println!("cargo:rerun-if-changed=src/windows/dismissal_macos.m");
    println!("cargo:rerun-if-changed=src/exports/region_magnifier_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_gpu_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_gpu_pipeline_macos.m");
    println!("cargo:rerun-if-changed=src/exports/osc_gpu_macos_shader.h");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+action.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+callbacks.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+magnifier.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+editor.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+keyboard.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+label.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+osc.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+selection.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+workspace.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+layout.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos+zoom.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+appearance.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+desktop.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+input.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+magnifier.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+state.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+ruler.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+snapshot.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+ocr.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+ocr_cancel.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+ocr_toolbar.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos+ocr_toolbar_input.m");
    println!("cargo:rerun-if-changed=src/exports/screenshot_region_osc_macos_private.h");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_surface_macos_private.h");
    println!(
      "cargo:rerun-if-changed=src/exports/recording_preview_surface_macos_private_functions.h"
    );
    println!("cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos.h");
    println!(
      "cargo:rerun-if-changed=src/exports/cursor_export/gpu_compositor_macos_keyboard_types.h"
    );
    println!("cargo:rerun-if-changed=src/exports/recording_preview_reader_macos.m");
    println!("cargo:rerun-if-changed=src/exports/recording_preview_scrubber_macos.m");
    println!("cargo:rerun-if-changed=src/recording/platform/camera/confidence_scaler_macos.m");
    println!("cargo:rerun-if-changed=src/recording/platform/desktop_compositor_macos.m");
    println!("cargo:rerun-if-changed=src/ruler/cursor_guard_macos.m");
    cc::Build::new()
      .file("src/exports/cursor_export/gpu_compositor_macos.m")
      .file("src/exports/cursor_export/gpu_compositor_macos+presenter.m")
      .file("src/exports/cursor_export/gpu_compositor_macos+presenter_keyboard.m")
      .file("src/exports/cursor_export/gpu_compositor_macos_cursor_resources.m")
      .file("src/exports/cursor_export/gpu_compositor_macos_keyboard.m")
      .file("src/exports/cursor_export/gpu_compositor_macos_keyboard_artwork.m")
      .file("src/recording/platform/camera/confidence_scaler_macos.m")
      .file("src/recording/platform/desktop_compositor_macos.m")
      .file("src/ruler/cursor_guard_macos.m")
      .file("src/exports/recording_preview_reader_macos.m")
      .file("src/exports/recording_preview_scrubber_macos.m")
      .file("src/exports/recording_preview_surface_macos.m")
      .file("src/exports/osc_material_surface_macos.m")
      .file("src/exports/osc_text_texture_macos.m")
      .file("src/exports/osc_icon_renderer_macos.m")
      .file("src/exports/osc_gpu_macos.m")
      .file("src/exports/osc_gpu_pipeline_macos.m")
      .file("src/exports/region_cursor_macos.m")
      .file("src/exports/cursor_session_macos.m")
      .file("src/windows/dismissal_macos.m")
      .file("src/exports/region_magnifier_macos.m")
      .file("src/exports/recording_preview_surface_macos+action.m")
      .file("src/exports/recording_preview_surface_macos+callbacks.m")
      .file("src/exports/recording_preview_surface_macos+magnifier.m")
      .file("src/exports/recording_preview_surface_macos+editor.m")
      .file("src/exports/recording_preview_surface_macos+keyboard.m")
      .file("src/exports/recording_preview_surface_macos+label.m")
      .file("src/exports/recording_preview_surface_macos+osc.m")
      .file("src/exports/recording_preview_surface_macos+selection.m")
      .file("src/exports/recording_preview_surface_macos+workspace.m")
      .file("src/exports/recording_preview_surface_macos+layout.m")
      .file("src/exports/recording_preview_surface_macos+zoom.m")
      .file("src/exports/screenshot_region_osc_macos.m")
      .file("src/exports/screenshot_region_osc_macos+appearance.m")
      .file("src/exports/screenshot_region_osc_macos+desktop.m")
      .file("src/exports/screenshot_region_osc_macos+input.m")
      .file("src/exports/screenshot_region_osc_macos+magnifier.m")
      .file("src/exports/screenshot_region_osc_macos+state.m")
      .file("src/exports/screenshot_region_osc_macos+ruler.m")
      .file("src/exports/screenshot_region_osc_macos+snapshot.m")
      .file("src/exports/screenshot_region_osc_macos+ocr.m")
      .file("src/exports/screenshot_region_osc_macos+ocr_cancel.m")
      .file("src/exports/screenshot_region_osc_macos+ocr_toolbar.m")
      .file("src/exports/screenshot_region_osc_macos+ocr_toolbar_input.m")
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
      "Metal",
      "QuartzCore",
      "VideoToolbox",
    ] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
    // Glide's tap recognition reads raw trackpad contact frames, which only
    // the private MultitouchSupport framework exposes.
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=MultitouchSupport");
  }
  tauri_build::build()
}

#[cfg(windows)]
fn compile_windows_preview_shaders() {
  compile_shader(
    "src/exports/preview_platform/surface_windows/shaders/preview.hlsl",
    "recording_preview",
  );
  compile_shader("src/osc/gpu/windows/shaders/osc.hlsl", "osc_gpu");
  compile_shader(
    "src/recording/platform_windows/shaders/desktop_compositor.hlsl",
    "desktop_compositor",
  );
}

#[cfg(windows)]
fn compile_shader(source_path: &str, output_prefix: &str) {
  use std::{ffi::CString, path::PathBuf};
  use windows::{
    core::PCSTR,
    Win32::Graphics::Direct3D::{Fxc::D3DCompile, ID3DBlob},
  };

  println!("cargo:rerun-if-changed={source_path}");
  let source = std::fs::read(source_path).expect("read the Windows preview shader");
  let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR"));

  for (entry, target, suffix) in [("vs_main", "vs_4_0", "vs"), ("ps_main", "ps_4_0", "ps")] {
    let entry = CString::new(entry).expect("valid shader entry");
    let target = CString::new(target).expect("valid shader target");
    let mut code: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;
    let result = unsafe {
      D3DCompile(
        source.as_ptr().cast(),
        source.len(),
        PCSTR::null(),
        None,
        None,
        PCSTR(entry.as_ptr().cast()),
        PCSTR(target.as_ptr().cast()),
        0,
        0,
        &mut code,
        Some(&mut errors),
      )
    };
    if let Err(error) = result {
      let detail = errors.map_or_else(String::new, |blob| unsafe {
        let bytes =
          std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize());
        String::from_utf8_lossy(bytes)
          .trim_matches(char::from(0))
          .to_owned()
      });
      panic!(
        "Windows preview shader compilation failed for {source_path} ({entry:?}, {target:?}): {error}: {detail}"
      );
    }
    let code = code.expect("D3DCompile returned preview bytecode");
    let bytes = unsafe {
      std::slice::from_raw_parts(code.GetBufferPointer().cast::<u8>(), code.GetBufferSize())
    };
    std::fs::write(output.join(format!("{output_prefix}_{suffix}.cso")), bytes)
      .expect("write compiled preview shader");
  }
}
