// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shaders compiled here rather than on the user's machine: to DXIL for
//! Windows and to Metal libraries for macOS. On Windows nothing keeps DXC's
//! output, so a shader compiled at runtime is compiled again every time the
//! device opens: the canvas shader took about 7 and 30 seconds over its two
//! variants, the annotate overlay's about 1.5. On macOS the system keeps what
//! Metal compiles, so the gain is smaller: about 0.4 seconds before a first
//! launch's first frame, and about 15 ms per shader on every later launch.
//! The GPU driver's own pass still runs at runtime, and the driver keeps
//! what that makes.
//!
//! A macOS build needs Apple's Metal compiler, which Xcode 26 leaves out
//! until `xcodebuild -downloadComponent MetalToolchain` downloads it; a
//! Windows build needs the DXC `pnpm dxc:prepare` places.
//!
//! Every compile is cached under `target/<triple>/shader-cache`, keyed by
//! all that goes into it, so a build that changes no shader compiles nothing.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
mod dxil;
#[cfg(target_os = "macos")]
mod metallib;

/// What a binding in group 0 holds, which decides the register or slot
/// wgpu gives it.
#[derive(Clone, Copy)]
pub enum Binding {
  Uniform,
  /// A read-only storage buffer.
  Storage,
  /// A filterable float 2D texture.
  Texture,
  /// A filterable float 2D array texture.
  TextureArray,
  /// A filtering sampler.
  Sampler,
}

/// A shader to compile: its `vs_main` and `fs_main`, drawn with one bind
/// group that only `fs_main` sees.
pub struct Precompiled<'a> {
  /// The constant the generated file declares for it.
  pub name: &'a str,
  pub source: String,
  /// Group 0's bindings, each at its index. The pipeline's layout has to be
  /// built from the same list, as the compiled stages expect wgpu's
  /// registers for exactly that layout.
  pub bindings: Vec<(u32, Binding)>,
}

/// A shader's compiled stages, as files.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
struct Stages {
  vertex: PathBuf,
  fragment: PathBuf,
}

/// Compiles each of `shaders` for the target and writes `OUT_DIR/<file>`,
/// for the crate to `include!`: a constant of type
/// `Option<screenwide_gpu::PrecompiledStages>` per shader, `None` where
/// nothing was compiled for the target.
pub fn precompile(file: &str, shaders: &[Precompiled]) {
  let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR"));
  let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  let cache = out_dir
    .ancestors()
    .nth(4)
    .unwrap_or(&out_dir)
    .join("shader-cache");
  std::fs::create_dir_all(&cache)
    .unwrap_or_else(|error| panic!("create {}: {error}", cache.display()));
  let compiled = if target == std::env::consts::OS {
    compile_for_host(shaders, &cache)
  } else {
    if target == "windows" || target == "macos" {
      println!(
        "cargo:warning=Shaders are not precompiled when cross-compiling for {target}; they \
         compile at runtime instead"
      );
    }
    shaders.iter().map(|_| None).collect()
  };
  let mut generated = String::from("// Written by screenwide_wgsl::precompile.\n");
  for (shader, stages) in shaders.iter().zip(compiled) {
    let value = stages.map_or_else(
      || "None".to_owned(),
      |stages| {
        format!(
          "Some(screenwide_gpu::PrecompiledStages {{ vertex: include_bytes!({:?}), fragment: \
           include_bytes!({:?}) }})",
          stages.vertex.display().to_string(),
          stages.fragment.display().to_string(),
        )
      },
    );
    generated.push_str(&format!(
      "const {}: Option<screenwide_gpu::PrecompiledStages> = {value};\n",
      shader.name
    ));
  }
  std::fs::write(out_dir.join(file), generated)
    .unwrap_or_else(|error| panic!("write {file}: {error}"));
}

#[cfg(target_os = "windows")]
fn compile_for_host(shaders: &[Precompiled], cache: &Path) -> Vec<Option<Stages>> {
  let compiler = dxil::Compiler::bundled();
  in_parallel(shaders, |shader| compiler.compile(shader, cache))
}

#[cfg(target_os = "macos")]
fn compile_for_host(shaders: &[Precompiled], cache: &Path) -> Vec<Option<Stages>> {
  let compiler = metallib::Compiler::installed();
  in_parallel(shaders, |shader| compiler.compile(shader, cache))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn compile_for_host(shaders: &[Precompiled], _cache: &Path) -> Vec<Option<Stages>> {
  shaders.iter().map(|_| None).collect()
}

/// Each shader compiled on a thread of its own: one can take far longer than
/// the rest, which are done in its shadow.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn in_parallel(
  shaders: &[Precompiled],
  compile: impl Fn(&Precompiled) -> Stages + Sync,
) -> Vec<Option<Stages>> {
  std::thread::scope(|scope| {
    let jobs: Vec<_> = shaders
      .iter()
      .map(|shader| scope.spawn(|| Some(compile(shader))))
      .collect();
    jobs
      .into_iter()
      .map(|job| job.join().expect("a shader compile panicked"))
      .collect()
  })
}

/// `shader` parsed and validated.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn parse(shader: &Precompiled) -> (naga::Module, naga::valid::ModuleInfo) {
  let module = naga::front::wgsl::parse_str(&shader.source)
    .unwrap_or_else(|error| panic!("the {} shader does not parse: {error}", shader.name));
  let info = naga::valid::Validator::new(
    naga::valid::ValidationFlags::all(),
    naga::valid::Capabilities::all(),
  )
  .validate(&module)
  .unwrap_or_else(|error| panic!("the {} shader does not validate: {error}", shader.name));
  (module, info)
}

/// `bindings` in the order wgpu numbers them: by index.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn in_binding_order(bindings: &[(u32, Binding)]) -> Vec<(u32, Binding)> {
  let mut ordered = bindings.to_vec();
  ordered.sort_by_key(|(index, _)| *index);
  ordered
}

/// The file `key` names in `cache`, made by `compile` unless an earlier build
/// made it. `compile` writes to the path it is given, which is moved into
/// place once complete, so an interrupted build leaves nothing behind.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
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
