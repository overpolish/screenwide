// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// On macOS the engine loads Dawn, WebGPU's library, at launch. It sits beside
// the helper in development and in tests, and in the bundle's Frameworks
// folder next to the MacOS folder the helper is installed in. ONNX Runtime's
// CoreML code checks the system's version with `__isPlatformVersionAtLeast`,
// which lives in clang's own runtime library; ort-sys linked that library
// itself until 2.0.0-rc.15, so it is linked here.
//
// On Windows transcribe.cpp asks for vulkan-1.lib but, unlike whisper-rs,
// does not tell the linker where the Vulkan SDK keeps it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  if target_os == "macos" && std::env::var_os("CARGO_FEATURE_ENGINE").is_some() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    let runtime = clang_runtime_dir();
    println!("cargo:rustc-link-search=native={}", runtime.display());
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
  }
  if target_os == "windows" && std::env::var_os("CARGO_FEATURE_DICTATION").is_some() {
    println!("cargo:rerun-if-env-changed=VULKAN_SDK");
    let sdk = std::env::var_os("VULKAN_SDK")
      .expect("dictation's helper needs the Vulkan SDK, with VULKAN_SDK set");
    println!(
      "cargo:rustc-link-search=native={}",
      Path::new(&sdk).join("Lib").display()
    );
  }
}

/// Where the C compiler keeps its runtime libraries for macOS.
fn clang_runtime_dir() -> PathBuf {
  println!("cargo:rerun-if-env-changed=CC");
  println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
  let compiler = std::env::var("CC").unwrap_or_else(|_| "clang".to_owned());
  let output = Command::new(&compiler)
    .arg("--print-resource-dir")
    .output()
    .unwrap_or_else(|error| panic!("could not run {compiler}: {error}"));
  assert!(
    output.status.success(),
    "{compiler} --print-resource-dir failed"
  );
  let dir = Path::new(String::from_utf8_lossy(&output.stdout).trim()).join("lib/darwin");
  assert!(
    dir.join("libclang_rt.osx.a").is_file(),
    "clang's runtime library is not in {}",
    dir.display()
  );
  dir
}
