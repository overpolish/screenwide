// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! DXIL for Windows. naga writes each stage as wgpu-hal 30 writes it for the
//! preview's pipeline layout (`create_pipeline_layout` in its
//! `dx12/device.rs`): the same registers and the same options, so the text is
//! the text wgpu would compile. The bundled DXC then compiles it with the
//! arguments wgpu passes. `compositor/precompiled_tests.rs` draws with the
//! result and with the shader compiled from WGSL, and requires the same pixels.

use std::path::{Path, PathBuf};
use std::process::Command;

use naga::back::hlsl;

use super::preview_bindings::{PreviewBinding, PREVIEW_BINDINGS};
use super::{cached, Stages};

/// The oldest shader model DXC compiles for, which every GPU wgpu drives
/// through DXC supports; nothing in the canvas shader needs a newer one.
const SHADER_MODEL: hlsl::ShaderModel = hlsl::ShaderModel::V6_0;
/// What wgpu passes DXC besides the entry point and profile.
const ARGUMENTS: [&str; 4] = ["-HV", "2018", "-no-warnings", "-Ges"];

pub(super) struct Compiler {
  executable: PathBuf,
  /// What decides the output besides the text: the compiler's own bytes.
  identity: u64,
}

impl Compiler {
  /// The DXC `pnpm dxc:prepare` places beside the bundled library.
  pub(super) fn bundled() -> Self {
    let binaries = Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplied it"))
      .join("binaries");
    let executable = binaries.join("dxc.exe");
    let library = binaries.join("dxcompiler.dll");
    let mut identity = std::hash::DefaultHasher::new();
    for path in [&executable, &library] {
      println!("cargo:rerun-if-changed={}", path.display());
      let bytes = std::fs::read(path).unwrap_or_else(|_| {
        panic!(
          "The shader compiler {} is missing; run `pnpm dxc:prepare`",
          path.display()
        )
      });
      std::hash::Hash::hash(&bytes, &mut identity);
    }
    Self {
      executable,
      identity: std::hash::Hasher::finish(&identity),
    }
  }

  pub(super) fn compile(&self, source: &str, cache: &Path) -> Stages {
    let module = naga::front::wgsl::parse_str(source).expect("the preview shader parses");
    let info = naga::valid::Validator::new(
      naga::valid::ValidationFlags::all(),
      naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("the preview shader validates");
    let options = options();
    let stage = |stage, entry: &str, profile: &str| {
      let text = hlsl_text(&module, &info, stage, entry, &options);
      cached(
        cache,
        (&text, entry, profile, ARGUMENTS, self.identity),
        "dxil",
        |output| self.run(&text, entry, profile, output),
      )
    };
    Stages {
      vertex: stage(naga::ShaderStage::Vertex, "vs_main", "vs_6_0"),
      fragment: stage(naga::ShaderStage::Fragment, "fs_main", "ps_6_0"),
    }
  }

  fn run(&self, text: &str, entry: &str, profile: &str, output: &Path) {
    let input = output.with_extension("hlsl");
    std::fs::write(&input, text)
      .unwrap_or_else(|error| panic!("write {}: {error}", input.display()));
    let result = Command::new(&self.executable)
      .args(["-E", entry, "-T", profile])
      .args(ARGUMENTS)
      .arg("-Fo")
      .arg(output)
      .arg(&input)
      .output()
      .unwrap_or_else(|error| panic!("run {}: {error}", self.executable.display()));
    let _ = std::fs::remove_file(&input);
    assert!(
      result.status.success(),
      "DXC could not compile the preview shader's {entry}:\n{}",
      String::from_utf8_lossy(&result.stderr)
    );
  }
}

/// `entry`'s HLSL as wgpu-hal writes it: overrides processed for that entry
/// point, and a vertex stage trimmed to what the fragment stage reads.
fn hlsl_text(
  module: &naga::Module,
  info: &naga::valid::ModuleInfo,
  stage: naga::ShaderStage,
  entry: &str,
  options: &hlsl::Options,
) -> String {
  let fragment = (stage == naga::ShaderStage::Vertex)
    .then(|| hlsl::FragmentEntryPoint::new(module, "fs_main").expect("fs_main exists"));
  let (module, info) = naga::back::pipeline_constants::process_overrides(
    module,
    info,
    Some((stage, entry)),
    &Default::default(),
  )
  .expect("the preview shader's overrides resolve");
  let mut text = String::new();
  hlsl::Writer::new(
    &mut text,
    options,
    &hlsl::PipelineOptions {
      entry_point: Some((stage, entry.to_owned())),
    },
  )
  .write(&module, &info, fragment.as_ref())
  .unwrap_or_else(|error| panic!("naga could not write the preview shader's {entry}: {error}"));
  text
}

fn target(space: u8, register: u32) -> hlsl::BindTarget {
  hlsl::BindTarget {
    space,
    register,
    binding_array_size: None,
    dynamic_storage_buffer_offsets_index: None,
    restrict_indexing: false,
  }
}

/// The options wgpu-hal 30 derives from the preview's layout: one bind group,
/// no immediates and no dynamic offsets. Uniforms take consecutive CBVs, and
/// textures and storage buffers consecutive SRVs, in binding order; samplers
/// index the sampler heap through a buffer in the SRV after them; the special
/// constants every wgpu pipeline layout carries take the CBV after the
/// uniforms.
fn options() -> hlsl::Options {
  let mut binding_map = hlsl::BindingMap::default();
  let (mut cbv, mut srv, mut sampler) = (0, 0, 0);
  for (kind, binding) in PREVIEW_BINDINGS.iter().zip(0..) {
    let resource = naga::ResourceBinding { group: 0, binding };
    match kind {
      PreviewBinding::Uniform => {
        binding_map.insert(resource, target(0, cbv));
        cbv += 1;
      }
      PreviewBinding::Storage | PreviewBinding::Texture | PreviewBinding::TextureArray => {
        binding_map.insert(resource, target(0, srv));
        srv += 1;
      }
      PreviewBinding::Sampler => {
        // naga does not read a sampler's space.
        binding_map.insert(resource, target(255, sampler));
        sampler += 1;
      }
    }
  }
  let mut sampler_buffer_binding_map = hlsl::SamplerIndexBufferBindingMap::default();
  if sampler > 0 {
    sampler_buffer_binding_map.insert(hlsl::SamplerIndexBufferKey { group: 0 }, target(0, srv));
  }
  hlsl::Options {
    shader_model: SHADER_MODEL,
    binding_map,
    fake_missing_bindings: false,
    special_constants_binding: Some(target(0, cbv)),
    immediates_target: None,
    dynamic_storage_buffer_offsets_targets: Default::default(),
    zero_initialize_workgroup_memory: true,
    restrict_indexing: true,
    sampler_heap_target: hlsl::SamplerHeapBindTargets {
      standard_samplers: target(0, 0),
      comparison_samplers: target(0, 2048),
    },
    sampler_buffer_binding_map,
    external_texture_binding_map: Default::default(),
    force_loop_bounding: true,
    // Read only by mesh and task shaders.
    task_dispatch_limits: None,
    mesh_shader_primitive_indices_clamp: true,
    ray_query_initialization_tracking: true,
  }
}
