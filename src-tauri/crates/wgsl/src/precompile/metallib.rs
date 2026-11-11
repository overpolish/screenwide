// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Metal libraries for macOS. naga writes each entry point as wgpu-hal 30
//! writes it for the shader's pipeline layout (`create_pipeline_layout` and
//! `load_shader` in its `metal/device.rs`): the same slots and the same
//! options, but for a fixed language version and without buffer bounds
//! checks (see [`resources`]). Apple's Metal compiler then compiles it as
//! wgpu would through `MTLCompileOptions`.
//!
//! The compiler is not part of Xcode's default install: Xcode 26 downloads it
//! with `xcodebuild -downloadComponent MetalToolchain`.

use std::path::Path;
use std::process::{Command, Output};

use naga::back::msl;

use super::{cached, entry_point, parse, stages_of, Compiled, Precompiled};

mod slots;
use slots::{resources, vertex_buffers};

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

  pub(super) fn compile(&self, shader: &Precompiled, cache: &Path) -> Vec<Compiled> {
    let (module, info) = parse(shader);
    shader
      .entry_points
      .iter()
      .map(|&name| {
        let (_, entry) = entry_point(shader, &module, name);
        let text = msl_text(shader, &module, &info, entry);
        let code = cached(
          cache,
          (&text, ARGUMENTS, &self.identity),
          "metallib",
          |output| self.run(shader.name, &text, name, output),
        );
        Compiled {
          name: name.to_owned(),
          workgroup_size: entry.workgroup_size,
          code,
        }
      })
      .collect()
  }

  fn run(&self, name: &str, text: &str, entry: &str, output: &Path) {
    let input = output.with_extension("metal");
    let air = output.with_extension("air");
    std::fs::write(&input, text)
      .unwrap_or_else(|error| panic!("write {}: {error}", input.display()));
    let succeeded = |result: std::io::Result<Output>, what: &str| {
      let result = result.unwrap_or_else(|error| panic!("{TOOLCHAIN_MISSING}: {error}"));
      assert!(
        result.status.success(),
        "{what} the {name} shader's {entry}:\n{}",
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

/// An entry point's MSL as wgpu-hal writes it: overrides processed for that
/// entry point, only its own resources mapped, and a vertex stage pulling
/// its vertices from the buffers wgpu binds them to. wgpu looks the function
/// up by the name the pipeline gives, so naga must keep it.
fn msl_text(
  shader: &Precompiled,
  module: &naga::Module,
  info: &naga::valid::ModuleInfo,
  entry: &naga::EntryPoint,
) -> String {
  let (name, stage) = (&entry.name, entry.stage);
  let (module, info) = naga::back::pipeline_constants::process_overrides(
    module,
    info,
    Some((stage, name)),
    &Default::default(),
  )
  .unwrap_or_else(|error| {
    panic!(
      "the {} shader's overrides do not resolve: {error}",
      shader.name
    )
  });
  let checked = naga::proc::BoundsCheckPolicy::Restrict;
  let options = msl::Options {
    lang_version: LANGUAGE_VERSION,
    per_entry_point_map: [(name.clone(), resources(shader.layout, stages_of(stage)))].into(),
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
  };
  let vertex_buffers = if stage == naga::ShaderStage::Vertex {
    vertex_buffers(shader.layout)
  } else {
    Vec::new()
  };
  let pulls_vertices = !vertex_buffers.is_empty();
  let (text, translation) = msl::write_string(
    &module,
    &info,
    &options,
    &msl::PipelineOptions {
      entry_point: Some((stage, name.clone())),
      allow_and_force_point_size: false,
      vertex_pulling_transform: true,
      vertex_buffer_mappings: vertex_buffers,
      binding_array_length_map: Default::default(),
    },
  )
  .unwrap_or_else(|error| {
    panic!(
      "naga could not write the {} shader's {name}: {error}",
      shader.name
    )
  });
  assert_eq!(
    translation.entry_point_names[0].as_deref().ok(),
    Some(name.as_str()),
    "naga renamed the {} shader's {name}",
    shader.name
  );
  // wgpu fills the sizes buffer with the lengths of the arrays it found
  // compiling MSL itself, then the vertex buffers' sizes; for a precompiled
  // library it finds none, so only the vertex buffers' are there, and only
  // if no array's length was meant to come before them.
  assert!(
    !text.contains("_buffer_sizes.size"),
    "the {} shader's {name} reads a buffer's length, which a precompiled Metal library cannot \
     be given",
    shader.name
  );
  let sizes = text
    .split("struct _mslBufferSizes {")
    .nth(1)
    .and_then(|rest| rest.split("};").next())
    .unwrap_or_default();
  assert!(
    !(pulls_vertices && sizes.contains("uint size")),
    "the {} shader's {name} pulls vertices past array lengths wgpu would not fill in",
    shader.name
  );
  text
}
