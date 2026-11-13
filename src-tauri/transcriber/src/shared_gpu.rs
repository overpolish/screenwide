// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A model sharing the GPU with whatever else the computer is doing, so work
//! in the background never takes a game or another program down with it.
//!
//! Each step runs on the GPU unless another program is using it heavily
//! (`others`); then it runs on the CPU, on half the processor, so the other
//! program keeps the GPU and most of the processor, and the GPU is used again
//! once it is quiet. A step the GPU fails is run again on the CPU, where the
//! model stays for the rest of the job. A GPU lost outright, as when Windows
//! resets it because one piece of work held it too long, takes every model
//! in the helper off it for the rest of the run, since the device is gone
//! for all of them, and is remembered against the model that lost it
//! (`resets`), so later jobs leave the GPU alone until the graphics driver
//! changes.

mod others;
mod resets;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ort::session::Session;

use crate::session::{on_cpu, on_cpu_threads};

/// The GPU was lost in this run of the helper.
static LOST: AtomicBool = AtomicBool::new(false);

/// What Windows answers once the GPU is gone: removed, hung or reset.
const DEVICE_LOST: [&str; 3] = ["887A0005", "887A0006", "887A0007"];

/// Where a step runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum On {
  Gpu,
  Cpu,
}

pub(crate) struct Shared {
  path: PathBuf,
  what: &'static str,
  gpu: Option<Session>,
  /// Loaded the first time a step runs on the CPU.
  cpu: Option<Session>,
  /// The threads the model takes on the CPU: half the processor for a model
  /// that had the GPU, ONNX Runtime's own choice for one that never did.
  cpu_threads: Option<usize>,
  /// The last step stepped aside for another program.
  aside: bool,
}

impl Shared {
  /// The model at `path` for the feature named `what`, loaded on the GPU by
  /// `open`, unless the GPU was lost already in this run or was reset by
  /// this model before under the graphics driver there is now.
  pub(crate) fn open(
    path: &Path,
    what: &'static str,
    open: impl FnOnce() -> Option<Session>,
  ) -> Self {
    let gpu = if LOST.load(Ordering::Relaxed) {
      None
    } else if resets::reset_before(path) {
      eprintln!("{what} runs on the CPU: this model has reset the GPU before");
      None
    } else {
      open()
    };
    let cpu_threads = gpu.is_some().then(half_the_processor);
    Self {
      path: path.to_owned(),
      what,
      gpu,
      cpu: None,
      cpu_threads,
      aside: false,
    }
  }

  /// The model at `path` already loaded on the CPU, in `session`.
  pub(crate) fn on_cpu(path: &Path, what: &'static str, session: Session) -> Self {
    Self {
      path: path.to_owned(),
      what,
      gpu: None,
      cpu: Some(session),
      cpu_threads: None,
      aside: false,
    }
  }

  pub(crate) fn has_gpu(&self) -> bool {
    self.gpu.is_some()
  }

  /// Runs `step` on the model where it should run now, telling it where, and
  /// again on the CPU where the GPU fails it. A step must change nothing
  /// outside itself until it succeeds, since a failed one runs again.
  pub(crate) fn run<T>(
    &mut self,
    mut step: impl FnMut(&mut Session, On) -> Result<T, String>,
  ) -> Result<T, String> {
    if LOST.load(Ordering::Relaxed) {
      self.gpu = None;
    }
    if let Some(gpu) = self.gpu.as_mut() {
      let busy = others::busy();
      if busy != self.aside {
        self.aside = busy;
        if busy {
          eprintln!(
            "{} moves to the CPU while another program uses the GPU",
            self.what
          );
        } else {
          eprintln!("{} moves back to the GPU", self.what);
        }
      }
      if !busy {
        match step(gpu, On::Gpu) {
          Ok(value) => return Ok(value),
          Err(error) => {
            eprintln!("{} moves to the CPU: {error}", self.what);
            if is_device_lost(&error) {
              LOST.store(true, Ordering::Relaxed);
              resets::remember(&self.path);
            }
            self.gpu = None;
          }
        }
      }
    }
    if self.cpu.is_none() {
      self.cpu = Some(match self.cpu_threads {
        Some(threads) => on_cpu_threads(&self.path, self.what, threads)?,
        None => on_cpu(&self.path, self.what)?,
      });
    }
    let Some(cpu) = self.cpu.as_mut() else {
      unreachable!("the CPU's model is loaded above")
    };
    step(cpu, On::Cpu)
  }
}

fn is_device_lost(error: &str) -> bool {
  let error = error.to_ascii_uppercase();
  DEVICE_LOST.iter().any(|code| error.contains(code))
}

fn half_the_processor() -> usize {
  std::thread::available_parallelism().map_or(2, |cores| (cores.get() / 2).max(1))
}

#[cfg(test)]
mod tests {
  use super::is_device_lost;

  #[test]
  fn only_a_lost_gpu_is_remembered() {
    // DirectML's own words for the reset this guards against.
    assert!(is_device_lost(
      "Exception(2) tid(73a4) 887A0006 The GPU will not respond to more commands"
    ));
    assert!(is_device_lost("HRESULT=0x887a0005"));
    // A wrong answer moves the job to the CPU without keeping the GPU from
    // the next one.
    assert!(!is_device_lost(
      "the GPU answered 1800 for audio peaking at 0.3"
    ));
  }
}
