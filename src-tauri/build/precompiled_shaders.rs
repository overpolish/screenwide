// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas shader's variant with no annotation and its variant with every
//! kind, compiled to DXIL here rather than on the user's machine: DXC took
//! about 7 seconds over the first and 30 over the second on every launch, as
//! nothing keeps its output. The GPU driver's own pass still runs at runtime,
//! and the driver keeps what that makes. macOS builds compile both at runtime,
//! as wgpu's Metal backend tells a precompiled library nothing of the buffer
//! sizes the shader's bounds checks read.
//!
//! Writes `OUT_DIR/precompiled_shaders.rs`, which
//! `src/editor/preview_platform/compositor/canvas_precompiled.rs` includes:
//! each variant's stages, or `None` where nothing was compiled for the
//! target. Every compile is cached under `target/shader-cache`, keyed by all
//! that goes into it, so a build that changes no shader compiles nothing.

#[cfg(target_os = "windows")]
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
#[path = "precompiled_shaders/dxil.rs"]
mod dxil;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[path = "../src/editor/preview_platform/compositor/preview_bindings.rs"]
mod preview_bindings;

/// The switch `compositor/canvas_variants.rs` turns off for the variant with
/// no annotation. The variant with every kind is the shader as written.
const DRAWN: &str = "const annotations_drawn: bool = true;";
const LEFT_OUT: &str = "const annotations_drawn: bool = false;";

/// A variant's compiled stages, as files.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
struct Stages {
  vertex: PathBuf,
  fragment: PathBuf,
}

pub(crate) fn precompile(out_dir: &Path) {
  let shader = std::fs::read_to_string(out_dir.join("preview.wgsl"))
    .unwrap_or_else(|error| panic!("read the assembled preview shader: {error}"));
  assert!(
    shader.contains(DRAWN),
    "preview.wgsl no longer declares `{DRAWN}`, which the variant with no annotation turns off"
  );
  let variants = [
    ("NO_ANNOTATIONS", shader.replacen(DRAWN, LEFT_OUT, 1)),
    ("EVERY_KIND", shader),
  ];
  let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  let cache = out_dir
    .ancestors()
    .nth(4)
    .unwrap_or(out_dir)
    .join("shader-cache");
  std::fs::create_dir_all(&cache)
    .unwrap_or_else(|error| panic!("create {}: {error}", cache.display()));
  let compiled = if target == std::env::consts::OS {
    compile_for_host(&variants, &cache)
  } else {
    if target == "windows" {
      println!(
        "cargo:warning=The canvas shader is not precompiled when cross-compiling for Windows; \
         it compiles at runtime instead"
      );
    }
    variants.iter().map(|_| None).collect()
  };
  let mut generated = String::from("// Written by build/precompiled_shaders.rs.\n");
  for ((name, _), stages) in variants.iter().zip(compiled) {
    let value = stages.map_or_else(
      || "None".to_owned(),
      |stages| {
        format!(
          "Some(Stages {{ vertex: include_bytes!({:?}), fragment: include_bytes!({:?}) }})",
          stages.vertex.display().to_string(),
          stages.fragment.display().to_string(),
        )
      },
    );
    generated.push_str(&format!("const {name}: Option<Stages> = {value};\n"));
  }
  std::fs::write(out_dir.join("precompiled_shaders.rs"), generated)
    .unwrap_or_else(|error| panic!("write precompiled_shaders.rs: {error}"));
}

#[cfg(target_os = "windows")]
fn compile_for_host(variants: &[(&str, String)], cache: &Path) -> Vec<Option<Stages>> {
  let compiler = dxil::Compiler::bundled();
  in_parallel(variants, |source| compiler.compile(source, cache))
}

#[cfg(not(target_os = "windows"))]
fn compile_for_host(variants: &[(&str, String)], _cache: &Path) -> Vec<Option<Stages>> {
  variants.iter().map(|_| None).collect()
}

/// Each variant compiled on a thread of its own: the one with every kind
/// takes far longer than the other, which is done in its shadow.
#[cfg(target_os = "windows")]
fn in_parallel(
  variants: &[(&str, String)],
  compile: impl Fn(&str) -> Stages + Sync,
) -> Vec<Option<Stages>> {
  std::thread::scope(|scope| {
    let jobs: Vec<_> = variants
      .iter()
      .map(|(_, source)| scope.spawn(|| Some(compile(source))))
      .collect();
    jobs
      .into_iter()
      .map(|job| job.join().expect("a shader compile panicked"))
      .collect()
  })
}

/// The file `key` names in `cache`, made by `compile` unless an earlier build
/// made it. `compile` writes to the path it is given, which is moved into
/// place once complete, so an interrupted build leaves nothing behind.
#[cfg(target_os = "windows")]
fn cached(cache: &Path, key: impl Hash, extension: &str, compile: impl FnOnce(&Path)) -> PathBuf {
  let mut hasher = DefaultHasher::new();
  key.hash(&mut hasher);
  let path = cache.join(format!("{:016x}.{extension}", hasher.finish()));
  if !path.exists() {
    let partial = path.with_extension(format!("{extension}.partial"));
    compile(&partial);
    std::fs::rename(&partial, &path)
      .unwrap_or_else(|error| panic!("move {} into place: {error}", path.display()));
  }
  path
}
