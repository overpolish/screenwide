// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The processes holding the library, a line each (`pid started`), written
//! before a process is touched and taken out once it is given back, so what
//! is on disk is always at least what is hidden.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use crate::HiddenProcess;

pub struct Journal {
  path: PathBuf,
  lock: Mutex<()>,
}

impl Journal {
  pub fn at(path: PathBuf) -> Self {
    Self {
      lock: Mutex::new(()),
      path,
    }
  }

  pub fn read(&self) -> Vec<HiddenProcess> {
    let _held = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
    self.entries()
  }

  pub fn add(&self, process: HiddenProcess) {
    let _held = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
    let mut entries = self.entries();
    if !entries.contains(&process) {
      entries.push(process);
    }
    self.write(&entries);
  }

  pub fn remove(&self, process: HiddenProcess) {
    let _held = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
    let mut entries = self.entries();
    entries.retain(|entry| *entry != process);
    self.write(&entries);
  }

  fn entries(&self) -> Vec<HiddenProcess> {
    std::fs::read_to_string(&self.path)
      .unwrap_or_default()
      .lines()
      .filter_map(|line| {
        let (pid, started) = line.split_once(' ')?;
        Some(HiddenProcess {
          pid: pid.parse().ok()?,
          started: started.parse().ok()?,
        })
      })
      .collect()
  }

  fn write(&self, entries: &[HiddenProcess]) {
    if entries.is_empty() {
      let _ = std::fs::remove_file(&self.path);
      return;
    }
    let text: String = entries
      .iter()
      .map(|entry| format!("{} {}\n", entry.pid, entry.started))
      .collect();
    if let Some(parent) = self.path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&self.path, text);
  }
}
