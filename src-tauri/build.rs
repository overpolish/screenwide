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
    // Headers the annotations crate owns, included by the sources below.
    println!("cargo:rerun-if-changed=crates/annotations/src/text/macos/annotation_text_box.h");
    println!("cargo:rerun-if-changed=crates/annotations/src/geometry.h");
    println!("cargo:rerun-if-changed=crates/annotations/src/reveal.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos.m");
    println!("cargo:rerun-if-changed=src/osc/cursor_macos/cursor_session_macos.m");
    println!("cargo:rerun-if-changed=src/app_windows/color_panel_macos.m");
    println!("cargo:rerun-if-changed=src/app_windows/dismissal_macos.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+callbacks.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+magnifier.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+crop_draw.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+framed.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+context_menu.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+drop.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_image_drop_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+cursor.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+editor.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+overlay.m");
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+workspace_move.m"
    );
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_text.m");
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_text_input.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_chrome.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_redact.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_image.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_picking.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+object_picking.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+object_picking_osc.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_state.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_report.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_group.m"
    );
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_annotation_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_annotation_geometry_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+osc.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+selection.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+turn.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_turn_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_selection_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+workspace.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+layout.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+editor_state.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+zoom.m");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon.m");
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon_draw.m"
    );
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon_state.m"
    );
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_audio_ribbon_macos.h");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+icon.m");
    // Headers the osc crate owns, included by the sources below.
    println!("cargo:rerun-if-changed=crates/osc/src/gpu/macos");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+draw.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+appearance.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+desktop.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+input.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+magnifier.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+state.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ruler.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+snapshot.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_cancel.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_toolbar.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_toolbar_input.m");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos_private.h");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos_types.h");
    println!("cargo:rerun-if-changed=src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos_private_functions.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos_private.h");
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos_private_functions.h"
    );
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_annotation_text_macos.h");
    println!("cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_annotation_group_macos.h");
    println!(
      "cargo:rerun-if-changed=src/editor/preview_platform/surface_macos/native/recording_preview_workspace_scene_macos.h"
    );
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
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos.m")
      .file("src/osc/cursor_macos/cursor_session_macos.m")
      .file("src/app_windows/color_panel_macos.m")
      .file("src/app_windows/dismissal_macos.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+callbacks.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+magnifier.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+crop_draw.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+framed.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+context_menu.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+drop.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+cursor.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+editor.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+overlay.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+workspace_move.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_picking.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+object_picking.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+object_picking_osc.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_text_input.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_text.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_chrome.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_redact.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_image.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_state.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_report.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+annotation_group.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+osc.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+selection.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+turn.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+workspace.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+layout.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+editor_state.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon_draw.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+audio_ribbon_state.m")
      .file("src/editor/preview_platform/surface_macos/native/recording_preview_surface_macos+zoom.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+icon.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+draw.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+appearance.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+desktop.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+input.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+magnifier.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+state.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ruler.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+snapshot.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_cancel.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_toolbar.m")
      .file("src/app_windows/screenshot_region/native_osc_macos/screenshot_region_osc_macos+ocr_toolbar_input.m")
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
