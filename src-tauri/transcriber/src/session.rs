// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a model runs: on the GPU when ONNX Runtime can reach it, several
//! times faster than the CPU and giving the same result, and on the CPU when
//! the GPU cannot take the model.

use std::path::Path;

use ort::ep::ExecutionProviderDispatch;
use ort::session::Session;

/// Loads the model at `path` for the feature named `what`, on the GPU if it
/// can be.
pub(crate) fn session(path: &Path, what: &str) -> Result<Session, String> {
  let name = path
    .file_name()
    .map_or_else(Default::default, |name| name.to_string_lossy());
  match gpu().map(|backend| on_gpu(path, backend)) {
    Some(Ok(session)) => {
      eprintln!("{what} runs {name} on the GPU through {BACKEND}");
      return Ok(session);
    }
    Some(Err(error)) => {
      eprintln!("{what} runs {name} on the CPU: {BACKEND} could not take it: {error}");
    }
    None => {}
  }
  on_cpu(path, what)
}

/// Loads the model at `path` for the feature named `what` on the CPU, for a
/// model a GPU backend takes but answers wrongly.
pub(crate) fn on_cpu(path: &Path, what: &str) -> Result<Session, String> {
  Session::builder()
    .and_then(|mut builder| builder.commit_from_file(path))
    .map_err(|error| format!("Could not load the {what} model: {error}"))
}

/// The model at `path` for the feature named `what` on the CPU with
/// `threads` of its own, for running several sessions side by side.
pub(crate) fn on_cpu_threads(path: &Path, what: &str, threads: usize) -> Result<Session, String> {
  let failed = |error: String| format!("Could not load the {what} model: {error}");
  let builder = Session::builder()
    .map_err(|error| failed(error.to_string()))?
    .with_intra_threads(threads)
    .map_err(|error| failed(error.to_string()))?;
  let mut builder = builder;
  builder
    .commit_from_file(path)
    .map_err(|error| failed(error.to_string()))
}

/// The model at `path` for the feature named `what` on the GPU through the
/// platform's own machine learning layer, CoreML on macOS and DirectML on
/// Windows, where ONNX Runtime can take it there, and on the CPU otherwise.
/// CoreML compiles a model on its first load, which takes a while; the
/// compiled model is kept in `cache` for every load after.
pub(crate) fn on_platform_gpu(path: &Path, what: &str, cache: &Path) -> Result<Session, String> {
  let name = path
    .file_name()
    .map_or_else(Default::default, |name| name.to_string_lossy());
  match platform_gpu(path, cache) {
    Some(Ok(session)) => return Ok(session),
    Some(Err(error)) => {
      eprintln!("{what} runs {name} on the CPU: the GPU could not take it: {error}")
    }
    None => {}
  }
  on_cpu(path, what)
}

#[cfg(target_os = "macos")]
fn platform_gpu(path: &Path, cache: &Path) -> Option<ort::Result<Session>> {
  use ort::ep::coreml::{ComputeUnits, ModelFormat};
  let _ = std::fs::create_dir_all(cache);
  let coreml = ort::ep::CoreML::default()
    .with_model_format(ModelFormat::MLProgram)
    .with_compute_units(ComputeUnits::CPUAndGPU)
    .with_model_cache_dir(cache.display())
    .build();
  let load = || -> ort::Result<Session> {
    Session::builder()?
      .with_execution_providers([coreml.error_on_failure()])?
      .commit_from_file(path)
  };
  Some(load())
}

#[cfg(windows)]
fn platform_gpu(path: &Path, _cache: &Path) -> Option<ort::Result<Session>> {
  gpu().map(|backend| on_gpu(path, backend))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_gpu(_path: &Path, _cache: &Path) -> Option<ort::Result<Session>> {
  None
}

fn on_gpu(path: &Path, backend: ExecutionProviderDispatch) -> ort::Result<Session> {
  let builder = Session::builder()?;
  // DirectML takes neither of these, and refuses the session if they are on.
  #[cfg(windows)]
  let builder = builder
    .with_memory_pattern(false)?
    .with_parallel_execution(false)?;
  builder
    .with_execution_providers([backend.error_on_failure()])?
    .commit_from_file(path)
}

#[cfg(target_os = "macos")]
const BACKEND: &str = "WebGPU";
#[cfg(windows)]
const BACKEND: &str = "DirectML";
#[cfg(not(any(target_os = "macos", windows)))]
const BACKEND: &str = "no GPU backend";

#[cfg(target_os = "macos")]
fn gpu() -> Option<ExecutionProviderDispatch> {
  Some(ort::ep::WebGPU::default().build())
}

#[cfg(windows)]
fn gpu() -> Option<ExecutionProviderDispatch> {
  Some(ort::ep::DirectML::default().build())
}

#[cfg(not(any(target_os = "macos", windows)))]
fn gpu() -> Option<ExecutionProviderDispatch> {
  None
}
