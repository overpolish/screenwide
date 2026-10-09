// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// On macOS the engine loads Dawn, WebGPU's library, at launch. It sits beside
// the helper in development and in tests, and in the bundle's Frameworks
// folder next to the MacOS folder the helper is installed in.

fn main() {
  let macos = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "macos");
  if macos && std::env::var_os("CARGO_FEATURE_ENGINE").is_some() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
  }
}
