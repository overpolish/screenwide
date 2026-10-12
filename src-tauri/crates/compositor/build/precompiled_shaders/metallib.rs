// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Metal libraries for macOS. naga writes each stage as wgpu-hal 30 writes it
//! for the preview's pipeline layout (`create_pipeline_layout` and
//! `load_shader` in its `metal/device.rs`): the same slots and the same
//! options, but for a fixed language version and without buffer bounds
//! checks (see [`options`]). Apple's Metal compiler then compiles it as wgpu
//! would through `MTLCompileOptions`. `compositor/precompiled_tests.rs` draws
//! with the result and with the shader compiled from WGSL, and requires the
//! same pixels.
//!
//! The compiler is not part of Xcode's default install: Xcode 26 downloads it
//! with `xcodebuild -downloadComponent MetalToolchain`.

use std::path::Path;
use std::process::{Command, Output};

use naga::back::msl;

use super::preview_bindings::{PreviewBinding, PREVIEW_BINDINGS};
use super::{cached, Stages};

/// The Metal Shading Language of macOS 14, the oldest `tauri.conf.json`
/// supports; wgpu itself picks the newest the running system has.
const LANGUAGE_VERSION: (u8, u8) = (3, 1);
/// What the Metal compiler is passed besides its input and output: the
/// language and system above, and the invariance wgpu turns on through
/// `MTLCompileOptions.preserveInvariance`. Fast math, wgpu's default too, is
/// the compiler's.
const ARGUMENTS: [&str; 3] = [
  "-std=metal3.1",
  "-mmacosx-version-min=14.2",
  "-fpreserve-invariance",
];
const TOOLCHAIN_MISSING: &str = "The Metal shader compiler is missing. Install Xcode 26 and run \
                                 `xcodebuild -downloadComponent MetalToolchain`";

pub(super) struct Compiler {
  /// What decides the output besides the text: both tools' versions.
  identity: String,
}

impl Compiler {
  pub(super) fn installed() -> Self {
    let version = |tool: &str| {
      let output = xcrun(tool, |command| command.arg("--version"))
        .unwrap_or_else(|error| panic!("{TOOLCHAIN_MISSING}: {error}"));
      assert!(
        output.status.success(),
        "{TOOLCHAIN_MISSING}:\n{}",
        String::from_utf8_lossy(&output.stderr)
      );
      // The rest names the toolchain's mount point, which changes on its own.
      String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
    };
    Self {
      identity: format!("{} / {}", version("metal"), version("metallib")),
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
    let stage = |stage, entry: &str| {
      let text = msl_text(&module, &info, stage, entry, &options);
      // naga still declares the sizes buffer and passes it down; reading a
      // field of it, as `arrayLength` does, would read what wgpu never binds.
      assert!(
        !text.contains("_buffer_sizes."),
        "the preview shader's {entry} reads a buffer's length, which a precompiled Metal \
         library cannot be given"
      );
      cached(
        cache,
        (&text, ARGUMENTS, &self.identity),
        "metallib",
        |output| self.run(&text, entry, output),
      )
    };
    Stages {
      vertex: stage(naga::ShaderStage::Vertex, "vs_main"),
      fragment: stage(naga::ShaderStage::Fragment, "fs_main"),
    }
  }

  fn run(&self, text: &str, entry: &str, output: &Path) {
    let input = output.with_extension("metal");
    let air = output.with_extension("air");
    std::fs::write(&input, text)
      .unwrap_or_else(|error| panic!("write {}: {error}", input.display()));
    let succeeded = |result: std::io::Result<Output>, what: &str| {
      let result = result.unwrap_or_else(|error| panic!("{TOOLCHAIN_MISSING}: {error}"));
      assert!(
        result.status.success(),
        "{what} the preview shader's {entry}:\n{}",
        String::from_utf8_lossy(&result.stderr)
      );
    };
    succeeded(
      xcrun("metal", |command| {
        command
          .args(ARGUMENTS)
          .arg("-c")
          .arg(&input)
          .arg("-o")
          .arg(&air)
      }),
      "The Metal compiler could not compile",
    );
    succeeded(
      xcrun("metallib", |command| {
        command.arg(&air).arg("-o").arg(output)
      }),
      "The Metal linker could not link",
    );
    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&air);
  }
}

fn xcrun(
  tool: &str,
  arguments: impl FnOnce(&mut Command) -> &mut Command,
) -> std::io::Result<Output> {
  let mut command = Command::new("xcrun");
  command.args(["-sdk", "macosx", tool]);
  arguments(&mut command).output()
}

/// `entry`'s MSL as wgpu-hal writes it: overrides processed for that entry
/// point, and only that entry point's resources mapped. wgpu looks the
/// function up by the name the pipeline gives, so naga must keep it.
fn msl_text(
  module: &naga::Module,
  info: &naga::valid::ModuleInfo,
  stage: naga::ShaderStage,
  entry: &str,
  options: &msl::Options,
) -> String {
  let (module, info) = naga::back::pipeline_constants::process_overrides(
    module,
    info,
    Some((stage, entry)),
    &Default::default(),
  )
  .expect("the preview shader's overrides resolve");
  let mut options = options.clone();
  options
    .per_entry_point_map
    .retain(|name, _| name.as_str() == entry);
  let (text, translation) = msl::write_string(
    &module,
    &info,
    &options,
    &msl::PipelineOptions {
      entry_point: Some((stage, entry.to_owned())),
      allow_and_force_point_size: false,
      vertex_pulling_transform: true,
      vertex_buffer_mappings: Vec::new(),
      binding_array_length_map: Default::default(),
    },
  )
  .unwrap_or_else(|error| panic!("naga could not write the preview shader's {entry}: {error}"));
  assert_eq!(
    translation.entry_point_names[0].as_deref().ok(),
    Some(entry),
    "naga renamed the preview shader's {entry}"
  );
  text
}

/// The slots wgpu-hal 30 derives from the preview's layout: one bind group,
/// seen by the fragment stage only, no immediates. Each stage counts buffers,
/// textures and samplers apart, in binding order, and takes the buffer after
/// its last for the sizes of runtime-sized arrays; the vertex stage takes it
/// always, for vertex pulling.
///
/// Buffer accesses are left unchecked, which wgpu does only when asked. A
/// checked access to a runtime-sized array (bindings 7, 8, 10, 11 and 14)
/// reads that array's length from the sizes buffer, and wgpu fills that
/// buffer from the `sized_bindings` it finds while compiling MSL itself; for
/// a precompiled library it has none, and binds nothing in that slot. naga
/// sets the policy for every buffer at once, so the uniform arrays in
/// bindings 0 and 1 go unchecked too. Indices into the shader's own arrays
/// and texel loads stay checked: neither reads the sizes buffer.
fn options() -> msl::Options {
  let mut fragment = msl::EntryPointResources::default();
  let (mut buffers, mut textures, mut samplers) = (0, 0, 0);
  for (kind, binding) in PREVIEW_BINDINGS.iter().zip(0..) {
    let mut target = msl::BindTarget::default();
    match kind {
      PreviewBinding::Uniform | PreviewBinding::Storage => {
        target.buffer = Some(buffers);
        buffers += 1;
      }
      PreviewBinding::Texture | PreviewBinding::TextureArray => {
        target.texture = Some(textures);
        textures += 1;
      }
      PreviewBinding::Sampler => {
        target.sampler = Some(msl::BindSamplerTarget::Resource(samplers));
        samplers += 1;
      }
    }
    fragment
      .resources
      .insert(naga::ResourceBinding { group: 0, binding }, target);
  }
  if PREVIEW_BINDINGS
    .iter()
    .any(|kind| matches!(kind, PreviewBinding::Storage))
  {
    fragment.sizes_buffer = Some(buffers);
  }
  let vertex = msl::EntryPointResources {
    sizes_buffer: Some(0),
    ..Default::default()
  };
  let checked = naga::proc::BoundsCheckPolicy::Restrict;
  msl::Options {
    lang_version: LANGUAGE_VERSION,
    per_entry_point_map: [
      ("vs_main".to_owned(), vertex),
      ("fs_main".to_owned(), fragment),
    ]
    .into(),
    inline_samplers: Vec::new(),
    spirv_cross_compatibility: false,
    fake_missing_bindings: false,
    bounds_check_policies: naga::proc::BoundsCheckPolicies {
      index: checked,
      buffer: naga::proc::BoundsCheckPolicy::Unchecked,
      image_load: checked,
      binding_array: naga::proc::BoundsCheckPolicy::Unchecked,
    },
    zero_initialize_workgroup_memory: true,
    force_loop_bounding: true,
    // Read only by mesh and task shaders.
    task_dispatch_limits: None,
    mesh_shader_primitive_indices_clamp: true,
    emit_int_div_checks: true,
    ray_query_initialization_tracking: true,
  }
}
