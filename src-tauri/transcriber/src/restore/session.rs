// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where Studio sound runs: on the GPU when ONNX Runtime can reach it, about
//! three times faster than the CPU and giving the same voice, and on the CPU
//! when the GPU cannot take the model.

use std::path::Path;

use ort::ep::ExecutionProviderDispatch;
use ort::session::Session;

/// Loads the model at `path`, on the GPU if it can be.
pub(super) fn session(path: &Path) -> Result<Session, String> {
  let name = path
    .file_name()
    .map_or_else(Default::default, |name| name.to_string_lossy());
  match gpu().map(|backend| on_gpu(path, backend)) {
    Some(Ok(session)) => {
      eprintln!("Studio sound runs {name} on the GPU through {BACKEND}");
      return Ok(session);
    }
    Some(Err(error)) => {
      eprintln!("Studio sound runs {name} on the CPU: {BACKEND} could not take it: {error}");
    }
    None => {}
  }
  Session::builder()
    .and_then(|mut builder| builder.commit_from_file(path))
    .map_err(|error| format!("Could not load the Studio sound model: {error}"))
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
