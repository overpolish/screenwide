// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tauri links the app manifest as a resource into binaries only, so the test
//! harness loads the system's Common Controls v5, which lacks the
//! `TaskDialogIndirect` that tauri imports, and Windows refuses to start it
//! (STATUS_ENTRYPOINT_NOT_FOUND). On MSVC the linker embeds the manifest into
//! every target instead, and Tauri's own copy is turned off so the two cannot
//! collide.

use std::path::Path;

pub fn windows_attributes() -> tauri_build::WindowsAttributes {
  if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
    return tauri_build::WindowsAttributes::new();
  }
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
  let manifest = Path::new(&manifest_dir).join("windows-app-manifest.xml");
  println!("cargo:rerun-if-changed={}", manifest.display());
  println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
  println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
  tauri_build::WindowsAttributes::new_without_app_manifest()
}
