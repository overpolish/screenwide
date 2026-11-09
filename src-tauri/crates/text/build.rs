// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Type is set by Core Text on macOS: the faces and the line widths the
    // Rust side sizes text from.
    println!("cargo:rerun-if-changed=src/engine/macos");
    let mut build = cc::Build::new();
    build
      .file("src/engine/macos/annotation_font.m")
      .file("src/engine/macos/annotation_text_box.m")
      .file("src/engine/macos/installed_fonts.m")
      .flag("-fobjc-arc");
    // The app finds Inter in its bundle. Tests and other debug programs run
    // outside one, so a debug build also knows the repository's copy.
    if std::env::var("PROFILE").as_deref() == Ok("debug") {
      let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplied it");
      let inter = std::path::Path::new(&manifest)
        .join("../../assets/Inter-VariableFont_opsz,wght.ttf")
        .display()
        .to_string();
      build.define("SCREENWIDE_DEBUG_INTER_PATH", format!("{inter:?}").as_str());
    }
    build.compile("screenwide_text_engine");
    for framework in ["AppKit", "CoreText"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
