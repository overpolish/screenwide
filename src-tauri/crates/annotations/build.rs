// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
  // Build scripts run on the host, so the target is read from Cargo rather
  // than `cfg!`: a Windows build from macOS must not compile Objective-C.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    // Annotation type is set by Core Text on macOS: the face and the line
    // widths the Rust side sizes text boxes from.
    println!("cargo:rerun-if-changed=src/text/macos");
    cc::Build::new()
      .file("src/text/macos/annotation_font.m")
      .file("src/text/macos/annotation_text_box.m")
      .flag("-fobjc-arc")
      .compile("screenwide_annotation_text");
    for framework in ["AppKit", "CoreText"] {
      println!("cargo:rustc-link-lib=framework={framework}");
    }
  }
}
