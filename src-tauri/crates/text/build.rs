// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Type is set by Core Text on macOS: the faces and the line widths the
    // Rust side sizes text from.
    println!("cargo:rerun-if-changed=src/engine/macos");
    cc::Build::new()
      .file("src/engine/macos/annotation_font.m")
      .file("src/engine/macos/annotation_text_box.m")
      .file("src/engine/macos/installed_fonts.m")
      .flag("-fobjc-arc")
      .compile("screenwide_text_engine");
    for framework in ["AppKit", "CoreText"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
