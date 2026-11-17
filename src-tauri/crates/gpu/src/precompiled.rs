// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shader and the layouts its pipelines are built with, both from one
//! [`ShaderLayout`]. Its code comes compiled from the build
//! (`screenwide_wgsl::precompile`): DXIL on Windows and Metal libraries on
//! macOS, handed to the device as they are, so only the GPU driver's own pass
//! is left, and the driver keeps what that makes. WGSL compiled at runtime
//! stands in only where the build compiled nothing: a build for another
//! platform, a variant the build does not make, a device that takes no
//! compiled shaders. On Windows that compile takes up to seconds and is done
//! again every time the device opens.

use std::borrow::Cow;
use std::cell::Cell;

use screenwide_shader_layout::ShaderLayout;

use crate::layouts::{bind_group_layout, vertex_attributes};

/// An entry point as the build compiled it.
pub struct PrecompiledEntryPoint {
  pub name: &'static str,
  pub workgroup_size: [u32; 3],
  pub code: &'static [u8],
}

/// A shader's entry points as the build compiled them against the layout
/// whose [`ShaderLayout::fingerprint`] this carries.
#[derive(Clone, Copy)]
pub struct PrecompiledShader {
  pub fingerprint: u64,
  pub entry_points: &'static [PrecompiledEntryPoint],
}

pub struct Shader {
  device: wgpu::Device,
  label: &'static str,
  bind_group_layouts: Vec<wgpu::BindGroupLayout>,
  pipeline_layout: wgpu::PipelineLayout,
  vertex_buffers: &'static [screenwide_shader_layout::VertexBuffer],
  modules: Modules,
}

enum Modules {
  Wgsl(wgpu::ShaderModule),
  /// One module per entry point, as the build compiles them.
  Precompiled(Vec<(&'static str, wgpu::ShaderModule)>),
}

/// A render pipeline drawn with one of a shader's vertex and fragment
/// stages.
pub struct RenderStages<'a> {
  pub label: &'a str,
  pub vertex: &'a str,
  pub fragment: &'a str,
  pub targets: &'a [Option<wgpu::ColorTargetState>],
  pub primitive: wgpu::PrimitiveState,
}

thread_local! {
  static WGSL_ONLY: Cell<bool> = const { Cell::new(false) };
}

/// Runs `make` with every shader it makes on this thread compiled from WGSL,
/// for tests that check a precompiled shader draws what its WGSL draws.
pub fn compiling_wgsl<R>(make: impl FnOnce() -> R) -> R {
  WGSL_ONLY.set(true);
  let made = make();
  WGSL_ONLY.set(false);
  made
}

impl Shader {
  /// `precompiled` where the build compiled it against `layout`, otherwise
  /// `wgsl` compiled now.
  pub fn new(
    device: &wgpu::Device,
    label: &'static str,
    layout: &ShaderLayout,
    wgsl: &str,
    precompiled: Option<PrecompiledShader>,
  ) -> Self {
    let bind_group_layouts = layout
      .groups
      .iter()
      .map(|entries| bind_group_layout(device, label, entries))
      .collect::<Vec<_>>();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(label),
      bind_group_layouts: &bind_group_layouts.iter().map(Some).collect::<Vec<_>>(),
      immediate_size: 0,
    });
    let precompiled = precompiled.filter(|precompiled| {
      // The build compiles each shader against the layout its constant is
      // named for; another layout here is a constant handed to the wrong
      // shader.
      debug_assert_eq!(
        precompiled.fingerprint,
        layout.fingerprint(),
        "the {label} shader was compiled against another layout"
      );
      precompiled.fingerprint == layout.fingerprint()
        && !WGSL_ONLY.get()
        && device
          .features()
          .contains(wgpu::Features::PASSTHROUGH_SHADERS)
    });
    let modules = match precompiled {
      Some(precompiled) => Modules::Precompiled(
        precompiled
          .entry_points
          .iter()
          .map(|entry_point| (entry_point.name, passthrough(device, label, entry_point)))
          .collect(),
      ),
      None => Modules::Wgsl(wgsl_module(device, label, wgsl)),
    };
    Self {
      device: device.clone(),
      label,
      bind_group_layouts,
      pipeline_layout,
      vertex_buffers: layout.vertex_buffers,
      modules,
    }
  }

  /// The layout bind group `group` is made against.
  pub fn bind_group_layout(&self, group: usize) -> &wgpu::BindGroupLayout {
    &self.bind_group_layouts[group]
  }

  pub fn render_pipeline(&self, stages: &RenderStages) -> wgpu::RenderPipeline {
    let attributes = self
      .vertex_buffers
      .iter()
      .map(vertex_attributes)
      .collect::<Vec<_>>();
    let buffers = self
      .vertex_buffers
      .iter()
      .zip(&attributes)
      .map(|(buffer, attributes)| {
        Some(wgpu::VertexBufferLayout {
          array_stride: buffer.stride,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes,
        })
      })
      .collect::<Vec<_>>();
    self
      .device
      .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(stages.label),
        layout: Some(&self.pipeline_layout),
        vertex: wgpu::VertexState {
          module: self.module(stages.vertex),
          entry_point: Some(stages.vertex),
          compilation_options: Default::default(),
          buffers: &buffers,
        },
        fragment: Some(wgpu::FragmentState {
          module: self.module(stages.fragment),
          entry_point: Some(stages.fragment),
          compilation_options: Default::default(),
          targets: stages.targets,
        }),
        primitive: stages.primitive,
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
  }

  pub fn compute_pipeline(&self, label: &str, entry_point: &str) -> wgpu::ComputePipeline {
    self
      .device
      .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&self.pipeline_layout),
        module: self.module(entry_point),
        entry_point: Some(entry_point),
        compilation_options: Default::default(),
        cache: None,
      })
  }

  fn module(&self, entry_point: &str) -> &wgpu::ShaderModule {
    match &self.modules {
      Modules::Wgsl(module) => module,
      Modules::Precompiled(modules) => {
        &modules
          .iter()
          .find(|(name, _)| *name == entry_point)
          .unwrap_or_else(|| {
            panic!(
              "the build did not compile {entry_point} of the {} shader",
              self.label
            )
          })
          .1
      }
    }
  }
}

/// The one place WGSL is compiled at runtime.
#[allow(clippy::disallowed_methods)]
fn wgsl_module(device: &wgpu::Device, label: &str, wgsl: &str) -> wgpu::ShaderModule {
  device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some(label),
    source: wgpu::ShaderSource::Wgsl(wgsl.into()),
  })
}

fn passthrough(
  device: &wgpu::Device,
  label: &str,
  entry_point: &PrecompiledEntryPoint,
) -> wgpu::ShaderModule {
  let [x, y, z] = entry_point.workgroup_size;
  #[cfg_attr(
    not(any(target_os = "macos", target_os = "windows")),
    allow(unused_mut, unused_variables)
  )]
  let mut descriptor = wgpu::ShaderModuleDescriptorPassthrough {
    label: Some(label),
    entry_points: Cow::Owned(vec![wgpu::PassthroughShaderEntryPoint {
      name: entry_point.name.into(),
      workgroup_size: (x, y, z),
    }]),
    ..Default::default()
  };
  #[cfg(target_os = "windows")]
  {
    descriptor.dxil = Some(Cow::Borrowed(entry_point.code));
  }
  #[cfg(target_os = "macos")]
  {
    descriptor.metallib = Some(Cow::Borrowed(entry_point.code));
  }
  // SAFETY: the build compiled this code from the shader's WGSL against the
  // layout whose fingerprint it carries, after checking that layout holds
  // every resource the entry point uses as the WGSL declares it, and
  // `Shader::new` builds the pipeline layout from that same layout. The Metal
  // libraries leave buffer accesses unchecked, as wgpu cannot give them
  // buffer lengths.
  unsafe { device.create_shader_module_passthrough(descriptor) }
}
