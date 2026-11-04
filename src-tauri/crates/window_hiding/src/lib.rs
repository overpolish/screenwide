// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeping the apps chosen in Settings out of captures on Windows while a
//! recording runs. A window is kept out of every capture only when its own
//! process asks, so Screenwide loads a small library into each running
//! process of a chosen app that owns a window (`window-hider`, beside the
//! app) and starts its exports there: one keeps each window's own setting
//! and hides it, and goes on hiding windows the app opens; the other gives
//! the settings back, after which the library is unloaded. The processes
//! hidden are written to a journal first, so a Screenwide that stops
//! without giving them back gives them back as it next starts.
//!
//! Hiding a window this way hides it from every capture, video calls'
//! included, for as long as the recording runs.

#[cfg(target_os = "windows")]
mod inject;
#[cfg(target_os = "windows")]
mod journal;
#[cfg(target_os = "windows")]
mod processes;

#[cfg(target_os = "windows")]
pub use journal::Journal;
#[cfg(target_os = "windows")]
pub use processes::{can_hide, image_of, window_owners};

/// A process holding the library, as the journal names it: its id and when
/// it started, so a later process given the same id is not taken for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HiddenProcess {
  pub pid: u32,
  pub started: u64,
}

/// Hides the windows of every process running from one of `images` (full
/// paths, lower case) that owns a window, recording each in `journal` before
/// it is touched. Processes that cannot be hidden are passed over. Returns
/// those hidden.
#[cfg(target_os = "windows")]
pub fn hide(images: &[String], library: &std::path::Path, journal: &Journal) -> Vec<HiddenProcess> {
  let mut hidden = Vec::new();
  for pid in window_owners() {
    if !image_of(pid).is_some_and(|image| images.contains(&image)) {
      continue;
    }
    let Some(process) = processes::started(pid).map(|started| HiddenProcess { pid, started })
    else {
      continue;
    };
    journal.add(process);
    if inject::call(pid, library, inject::Export::Hide).is_ok() {
      hidden.push(process);
    } else {
      // Nothing was changed there, or the library never loaded.
      let _ = inject::unload(pid, library);
      journal.remove(process);
    }
  }
  hidden
}

/// Gives `processes` their windows' settings back and unloads the library
/// from them, then takes them out of `journal`. One that has since quit, or
/// been replaced by another process with its id, is only taken out.
#[cfg(target_os = "windows")]
pub fn restore(processes: &[HiddenProcess], library: &std::path::Path, journal: &Journal) {
  for &process in processes {
    if processes::started(process.pid) == Some(process.started) {
      let _ = inject::call(process.pid, library, inject::Export::Restore);
      let _ = inject::unload(process.pid, library);
    }
    journal.remove(process);
  }
}

/// Gives back what a Screenwide that stopped while recording left hidden.
#[cfg(target_os = "windows")]
pub fn restore_left_hidden(library: &std::path::Path, journal: &Journal) {
  let left = journal.read();
  if !left.is_empty() {
    restore(&left, library, journal);
  }
}
