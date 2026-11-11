// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shaders compiled here rather than on the user's machine: to DXIL for
//! Windows and to Metal libraries for macOS. On Windows nothing keeps DXC's
//! output, so a shader compiled at runtime is compiled again every time the
//! device opens: the canvas shader took about 7 and 30 seconds over its two
//! variants, the annotate overlay's about 1.5. On macOS the system keeps what
//! Metal compiles, so a launch after the first pays little, but the first
//! one waits on every shader it draws with. The GPU driver's own pass still
//! runs at runtime, and the driver keeps what that makes.
//!
//! Each shader is compiled against its [`ShaderLayout`], the one
//! `screenwide_gpu::Shader` builds its pipelines from, and checked against
//! it first: every resource an entry point uses has to be in the layout, of
//! the kind the WGSL declares and seen by that entry point's stage.
//!
//! A macOS build needs Apple's Metal compiler, which Xcode 26 leaves out
//! until `xcodebuild -downloadComponent MetalToolchain` downloads it; a
//! Windows build needs the DXC `pnpm dxc:prepare` places.
//!
//! Every compile is cached under `target/<triple>/shader-cache`, keyed by
//! all that goes into it, so a build that changes no shader compiles nothing.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use screenwide_shader_layout::{ShaderLayout, Stages};

mod check;
#[cfg(target_os = "windows")]
mod dxil;
#[cfg(target_os = "macos")]
mod metallib;

/// A shader to compile.
pub struct Precompiled<'a> {
  /// The constant the generated file declares for it.
  pub name: &'a str,
  pub source: String,
  pub layout: &'a ShaderLayout,
  /// The entry points to compile. A vertex stage is drawn with any of the
  /// fragment stages listed beside it, which all have to read what it
  /// writes alike.
  pub entry_points: &'a [&'a str],
}

/// An entry point compiled to a file.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
struct Compiled {
  name: String,
  workgroup_size: [u32; 3],
  code: PathBuf,
}

/// Compiles each of `shaders` for the target and writes `OUT_DIR/<file>`,
/// for the crate to `include!`: a constant of type
/// `Option<screenwide_gpu::PrecompiledShader>` per shader, `None` where
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
  for shader in shaders {
    let (module, info) = parse(shader);
    check::layout(shader, &module, &info);
  }
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
  for (shader, entry_points) in shaders.iter().zip(compiled) {
    let value = entry_points.map_or_else(
      || "None".to_owned(),
      |entry_points| {
        let entry_points = entry_points
          .iter()
          .map(|compiled| {
            format!(
              "screenwide_gpu::PrecompiledEntryPoint {{ name: {:?}, workgroup_size: {:?}, code: \
               include_bytes!({:?}) }}",
              compiled.name,
              compiled.workgroup_size,
              compiled.code.display().to_string(),
            )
          })
          .collect::<Vec<_>>()
          .join(", ");
        format!(
          "Some(screenwide_gpu::PrecompiledShader {{ fingerprint: {:#x}, entry_points: &[{entry_points}] }})",
          shader.layout.fingerprint(),
        )
      },
    );
    generated.push_str(&format!(
      "pub(crate) const {}: Option<screenwide_gpu::PrecompiledShader> = {value};\n",
      shader.name
    ));
  }
  std::fs::write(out_dir.join(file), generated)
    .unwrap_or_else(|error| panic!("write {file}: {error}"));
}

#[cfg(target_os = "windows")]
fn compile_for_host(shaders: &[Precompiled], cache: &Path) -> Vec<Option<Vec<Compiled>>> {
  let compiler = dxil::Compiler::bundled();
  in_parallel(shaders, |shader| compiler.compile(shader, cache))
}

#[cfg(target_os = "macos")]
fn compile_for_host(shaders: &[Precompiled], cache: &Path) -> Vec<Option<Vec<Compiled>>> {
  let compiler = metallib::Compiler::installed();
  in_parallel(shaders, |shader| compiler.compile(shader, cache))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn compile_for_host(shaders: &[Precompiled], _cache: &Path) -> Vec<Option<Vec<Compiled>>> {
  shaders.iter().map(|_| None).collect()
}

/// Each shader compiled on a thread of its own: one can take far longer than
/// the rest, which are done in its shadow.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn in_parallel(
  shaders: &[Precompiled],
  compile: impl Fn(&Precompiled) -> Vec<Compiled> + Sync,
) -> Vec<Option<Vec<Compiled>>> {
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

/// The entry point `name` of `shader`.
fn entry_point<'m>(
  shader: &Precompiled,
  module: &'m naga::Module,
  name: &str,
) -> (usize, &'m naga::EntryPoint) {
  module
    .entry_points
    .iter()
    .enumerate()
    .find(|(_, entry)| entry.name == name)
    .unwrap_or_else(|| panic!("the {} shader has no entry point {name}", shader.name))
}

/// The stages that see a binding, as one entry point's stage.
fn stages_of(stage: naga::ShaderStage) -> Stages {
  match stage {
    naga::ShaderStage::Vertex => Stages::VERTEX,
    naga::ShaderStage::Fragment => Stages::FRAGMENT,
    naga::ShaderStage::Compute => Stages::COMPUTE,
    other => panic!("{other:?} shaders are not precompiled"),
  }
}

/// The file `key` names in `cache`, made by `compile` unless an earlier build
/// made it. `compile` writes to a path of its own, which is moved into place
/// once complete, so an interrupted build leaves nothing behind. Shaders
/// that share an entry point's text share its file, and the build scripts of
/// several crates and the threads of one may make it at once: each writes
/// its own, and whichever moves into place last stands, the same bytes.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
fn cached(cache: &Path, key: impl Hash, extension: &str, compile: impl FnOnce(&Path)) -> PathBuf {
  static COMPILES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
  let mut hasher = DefaultHasher::new();
  key.hash(&mut hasher);
  let path = cache.join(format!("{:016x}.{extension}", hasher.finish()));
  if !path.exists() {
    let compile_number = COMPILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let partial = path.with_extension(format!(
      "{extension}.{}-{compile_number}.partial",
      std::process::id()
    ));
    compile(&partial);
    if let Err(error) = std::fs::rename(&partial, &path) {
      assert!(path.exists(), "move {} into place: {error}", path.display());
      let _ = std::fs::remove_file(&partial);
    }
  }
  path
}
