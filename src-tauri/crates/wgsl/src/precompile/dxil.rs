// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! DXIL for Windows. naga writes each entry point as wgpu-hal 30 writes it
//! for the shader's pipeline layout (`create_pipeline_layout` and
//! `load_shader` in its `dx12/device.rs`): the same registers and the same
//! options, so the text is the text wgpu would compile. The bundled DXC then
//! compiles it with the arguments wgpu passes.

use std::path::{Path, PathBuf};
use std::process::Command;

use naga::back::hlsl;
use screenwide_shader_layout::{Binding, ShaderLayout};

use super::{cached, entry_point, parse, Compiled, Precompiled};

/// The oldest shader model DXC compiles for, which every GPU wgpu drives
/// through DXC supports; nothing in the precompiled shaders needs a newer
/// one.
const SHADER_MODEL: hlsl::ShaderModel = hlsl::ShaderModel::V6_0;
/// What wgpu passes DXC besides the entry point and profile.
const ARGUMENTS: [&str; 4] = ["-HV", "2018", "-no-warnings", "-Ges"];

pub(super) struct Compiler {
  executable: PathBuf,
  /// What decides the output besides the text: the compiler's own bytes.
  identity: u64,
}

impl Compiler {
  /// The DXC `pnpm dxc:prepare` places beside the bundled library, in the
  /// app's `binaries` two folders above this crate.
  pub(super) fn bundled() -> Self {
    let binaries = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../binaries");
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

  pub(super) fn compile(&self, shader: &Precompiled, cache: &Path) -> Vec<Compiled> {
    let (module, info) = parse(shader);
    let options = options(shader.layout);
    let fragments = shader
      .entry_points
      .iter()
      .copied()
      .filter(|&name| entry_point(shader, &module, name).1.stage == naga::ShaderStage::Fragment)
      .collect::<Vec<_>>();
    shader
      .entry_points
      .iter()
      .map(|&name| {
        let (_, entry) = entry_point(shader, &module, name);
        let text = |fragment: Option<&str>| {
          hlsl_text(shader.name, &module, &info, entry, fragment, &options)
        };
        let (profile, text) = match entry.stage {
          naga::ShaderStage::Vertex => ("vs_6_0", vertex_text(shader.name, name, &fragments, text)),
          naga::ShaderStage::Fragment => ("ps_6_0", text(None)),
          naga::ShaderStage::Compute => ("cs_6_0", text(None)),
          other => panic!("{other:?} shaders are not precompiled"),
        };
        let code = cached(
          cache,
          (&text, name, profile, ARGUMENTS, self.identity),
          "dxil",
          |output| self.run(shader.name, &text, name, profile, output),
        );
        Compiled {
          name: name.to_owned(),
          workgroup_size: entry.workgroup_size,
          code,
        }
      })
      .collect()
  }

  fn run(&self, name: &str, text: &str, entry: &str, profile: &str, output: &Path) {
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
      "DXC could not compile the {name} shader's {entry}:\n{}",
      String::from_utf8_lossy(&result.stderr)
    );
  }
}

/// A vertex stage's HLSL. wgpu trims it to what the pipeline's fragment
/// stage reads, so one compiled vertex stage serves every fragment stage only
/// if each trims it alike.
fn vertex_text(
  shader: &str,
  vertex: &str,
  fragments: &[&str],
  text: impl Fn(Option<&str>) -> String,
) -> String {
  let Some((first, rest)) = fragments.split_first() else {
    return text(None);
  };
  let trimmed = text(Some(first));
  for fragment in rest {
    assert!(
      text(Some(fragment)) == trimmed,
      "the {shader} shader's {vertex} writes what {first} and {fragment} read differently, so \
       one compiled vertex stage cannot serve both"
    );
  }
  trimmed
}

/// An entry point's HLSL as wgpu-hal writes it: overrides processed for that
/// entry point, and a vertex stage trimmed to what `fragment` reads.
fn hlsl_text(
  shader: &str,
  module: &naga::Module,
  info: &naga::valid::ModuleInfo,
  entry: &naga::EntryPoint,
  fragment: Option<&str>,
  options: &hlsl::Options,
) -> String {
  let name = &entry.name;
  let fragment = fragment.map(|fragment| {
    hlsl::FragmentEntryPoint::new(module, fragment)
      .unwrap_or_else(|| panic!("the {shader} shader has no fragment stage {fragment}"))
  });
  let (module, info) = naga::back::pipeline_constants::process_overrides(
    module,
    info,
    Some((entry.stage, name)),
    &Default::default(),
  )
  .unwrap_or_else(|error| panic!("the {shader} shader's overrides do not resolve: {error}"));
  let mut text = String::new();
  hlsl::Writer::new(
    &mut text,
    options,
    &hlsl::PipelineOptions {
      entry_point: Some((entry.stage, name.clone())),
    },
  )
  .write(&module, &info, fragment.as_ref())
  .unwrap_or_else(|error| panic!("naga could not write the {shader} shader's {name}: {error}"));
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

/// The options wgpu-hal 30 derives from a layout with no immediates. Each
/// group takes, in binding order, consecutive CBVs for its uniforms, SRVs
/// for its textures and read-only storage, and UAVs for what is written;
/// then its samplers, which index the sampler heap through a buffer in the
/// SRV after them; then a CBV for each dynamic uniform, a root descriptor of
/// its own. The counts run on from one group to the next, and the special
/// constants every wgpu pipeline layout carries take the CBV after them all.
fn options(layout: &ShaderLayout) -> hlsl::Options {
  let mut binding_map = hlsl::BindingMap::default();
  let mut sampler_buffer_binding_map = hlsl::SamplerIndexBufferBindingMap::default();
  let (mut cbv, mut srv, mut uav) = (0, 0, 0);
  for (group, entries) in (0..).zip(layout.groups.iter()) {
    let resource = |binding| naga::ResourceBinding { group, binding };
    for entry in entries.iter() {
      let register = match entry.kind {
        Binding::Uniform => &mut cbv,
        Binding::Storage { writable: false } | Binding::Texture { .. } => &mut srv,
        Binding::Storage { writable: true } | Binding::StorageTexture { .. } => &mut uav,
        Binding::DynamicUniform | Binding::Sampler { .. } => continue,
      };
      binding_map.insert(resource(entry.binding), target(0, *register));
      *register += 1;
    }
    let samplers = entries
      .iter()
      .filter(|entry| matches!(entry.kind, Binding::Sampler { .. }));
    let mut count = 0;
    for entry in samplers {
      // naga does not read a sampler's space.
      binding_map.insert(resource(entry.binding), target(255, count));
      count += 1;
    }
    if count > 0 {
      sampler_buffer_binding_map.insert(hlsl::SamplerIndexBufferKey { group }, target(0, srv));
      srv += 1;
    }
    let dynamic = entries
      .iter()
      .filter(|entry| entry.kind == Binding::DynamicUniform);
    for entry in dynamic {
      binding_map.insert(
        resource(entry.binding),
        hlsl::BindTarget {
          restrict_indexing: true,
          ..target(0, cbv)
        },
      );
      cbv += 1;
    }
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
