// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// On macOS the engine loads Dawn, WebGPU's library, at launch. It sits beside
// the helper in development and in tests, and in the bundle's Frameworks
// folder next to the MacOS folder the helper is installed in.
//
// On Windows transcribe.cpp asks for vulkan-1.lib but, unlike whisper-rs,
// does not tell the linker where the Vulkan SDK keeps it.

fn main() {
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  if target_os == "macos" && std::env::var_os("CARGO_FEATURE_ENGINE").is_some() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
  }
  if target_os == "windows" && std::env::var_os("CARGO_FEATURE_DICTATION").is_some() {
    println!("cargo:rerun-if-env-changed=VULKAN_SDK");
    let sdk = std::env::var_os("VULKAN_SDK")
      .expect("dictation's helper needs the Vulkan SDK, with VULKAN_SDK set");
    println!(
      "cargo:rustc-link-search=native={}",
      std::path::Path::new(&sdk).join("Lib").display()
    );
  }
}
