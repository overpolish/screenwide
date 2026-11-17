// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a model runs: on the GPU when ONNX Runtime can reach it, several
//! times faster than the CPU and giving the same result, and on the CPU when
//! the GPU cannot take the model.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use ort::environment::Environment;
use ort::ep::ExecutionProviderDispatch;
use ort::session::builder::{AutoDevicePolicy, SessionBuilder};
use ort::session::Session;
use screenwide_transcriber::compiled_model_prefix;

/// ONNX Runtime's environment, every session's. ort allows one a process,
/// and refuses a second even once the first is gone, so it is made once and
/// kept for the helper's life.
static ENVIRONMENT: LazyLock<Result<Environment, String>> =
  LazyLock::new(|| ort::init().build().map_err(|error| error.to_string()));

/// A session builder that places the model where it is told. ort's own
/// builder asks ONNX Runtime to pick the most efficient device for a session
/// given no backend, which on a Mac is CoreML on the Neural Engine, and that
/// computes global pooling wrongly: Background removal's matte came out with
/// the room in it. A session given no backend runs on the CPU instead.
fn builder() -> ort::Result<SessionBuilder> {
  match &*ENVIRONMENT {
    Ok(environment) => {
      Ok(Session::builder(environment)?.with_auto_device(AutoDevicePolicy::PreferCPU)?)
    }
    Err(error) => Err(ort::Error::new(error.clone())),
  }
}

/// Whether DirectML may fuse the model's graph into one piece of GPU work.
/// Fused, a large model can hold the GPU for long enough that Windows resets
/// it, as Studio ducking's did, taking every program on it along; unfused,
/// the same model ran faster in many short steps. Only DirectML reads this.
#[derive(Clone, Copy)]
pub(crate) enum Fusion {
  On,
  Off,
}

/// Loads the model at `path` for the feature named `what`, on the GPU if it
/// can be.
pub(crate) fn session(path: &Path, what: &str) -> Result<Session, String> {
  match gpu_session(path, what, Fusion::On) {
    Some(session) => Ok(session),
    None => on_cpu(path, what),
  }
}

/// The model at `path` for the feature named `what` on the GPU, or nothing
/// where ONNX Runtime cannot take it there.
pub(crate) fn gpu_session(path: &Path, what: &str, fusion: Fusion) -> Option<Session> {
  let name = path
    .file_name()
    .map_or_else(Default::default, |name| name.to_string_lossy());
  match on_gpu(path, gpu()?, fusion) {
    Ok(session) => {
      eprintln!("{what} runs {name} on the GPU through {BACKEND}");
      Some(session)
    }
    Err(error) => {
      eprintln!("{what} runs {name} on the CPU: {BACKEND} could not take it: {error}");
      None
    }
  }
}

/// Loads the model at `path` for the feature named `what` on the CPU, for a
/// model a GPU backend takes but answers wrongly.
pub(crate) fn on_cpu(path: &Path, what: &str) -> Result<Session, String> {
  builder()
    .and_then(|mut builder| builder.commit_from_file(path))
    .map_err(|error| format!("Could not load the {what} model: {error}"))
}

/// The model at `path` for the feature named `what` on the CPU with
/// `threads` of its own, for running several sessions side by side.
pub(crate) fn on_cpu_threads(path: &Path, what: &str, threads: usize) -> Result<Session, String> {
  let failed = |error: String| format!("Could not load the {what} model: {error}");
  let builder = builder()
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
/// Windows, or nothing where ONNX Runtime cannot take it there. CoreML
/// compiles a model on its first load, which takes a while; the compiled
/// model is kept in `cache` for every load after.
pub(crate) fn on_platform_gpu(path: &Path, what: &str, cache: &Path) -> Option<Session> {
  let name = path
    .file_name()
    .map_or_else(Default::default, |name| name.to_string_lossy());
  match platform_gpu(path, cache)? {
    Ok(session) => Some(session),
    Err(error) => {
      eprintln!("{what} runs {name} on the CPU: the GPU could not take it: {error}");
      None
    }
  }
}

/// Where CoreML keeps the `graphs` it compiles: beside them, in a folder
/// named after the first ([`compiled_model_prefix`]) and a hash of the ONNX
/// Runtime build that compiled them and each graph's size and modification
/// time. ONNX Runtime finds a compiled graph by its file's path alone, so one
/// compiled before a graph was replaced at that path, as a newer download
/// does, fails to run rather than being compiled again; and one compiled by
/// another build does too. The other folders for the same first graph,
/// compiled for what is gone, are taken away.
pub(crate) fn coreml_cache(graphs: &[&Path]) -> PathBuf {
  use std::hash::{Hash, Hasher};
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  ort::info().hash(&mut hasher);
  for graph in graphs {
    let made = std::fs::metadata(graph).ok();
    made.as_ref().map(std::fs::Metadata::len).hash(&mut hasher);
    made.and_then(|made| made.modified().ok()).hash(&mut hasher);
  }
  let first = graphs
    .first()
    .copied()
    .unwrap_or_else(|| Path::new("model"));
  let folder = first.parent().unwrap_or_else(|| Path::new("."));
  let prefix = compiled_model_prefix(&first.file_name().unwrap_or_default().to_string_lossy());
  let name = format!("{prefix}{:016x}", hasher.finish());
  if let Ok(entries) = std::fs::read_dir(folder) {
    for entry in entries.flatten() {
      let other = entry.file_name().to_string_lossy().into_owned();
      if other.starts_with(&prefix) && other != name {
        let _ = std::fs::remove_dir_all(entry.path());
      }
    }
  }
  folder.join(name)
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
    builder()?
      .with_execution_providers([coreml.error_on_failure()])?
      .commit_from_file(path)
  };
  Some(load())
}

#[cfg(windows)]
fn platform_gpu(path: &Path, _cache: &Path) -> Option<ort::Result<Session>> {
  gpu().map(|backend| on_gpu(path, backend, Fusion::On))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_gpu(_path: &Path, _cache: &Path) -> Option<ort::Result<Session>> {
  None
}

fn gpu_builder(backend: ExecutionProviderDispatch, fusion: Fusion) -> ort::Result<SessionBuilder> {
  let builder = builder()?;
  // DirectML takes neither of these, and refuses the session if they are on.
  #[cfg(windows)]
  let builder = builder
    .with_memory_pattern(false)?
    .with_parallel_execution(false)?;
  #[cfg(windows)]
  let builder = match fusion {
    Fusion::On => builder,
    Fusion::Off => builder.with_config_entry("ep.dml.disable_graph_fusion", "1")?,
  };
  #[cfg(not(windows))]
  let _ = fusion;
  Ok(builder.with_execution_providers([backend.error_on_failure()])?)
}

fn on_gpu(path: &Path, backend: ExecutionProviderDispatch, fusion: Fusion) -> ort::Result<Session> {
  gpu_builder(backend, fusion)?.commit_from_file(path)
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
