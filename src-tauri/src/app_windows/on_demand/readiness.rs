// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Handing a window built on demand to what waited for it: once its page is
//! ready, when it could not be built, or when it was shown again on its way
//! to being destroyed.

use tauri::{AppHandle, Manager, WebviewWindow};

use super::super::WindowLabel;
use super::{state, Stage};

pub(super) fn ready(app: &AppHandle, label: WindowLabel, build: Option<u64>) {
  let (waiting, frame) = {
    let mut state = state();
    match state.stages.get(&label) {
      Some(Stage::Loading { build: current, .. })
        if build.is_none_or(|build| build == *current) => {}
      _ => return,
    }
    let Some(Stage::Loading { waiting, .. }) = state.stages.insert(label, Stage::Ready) else {
      return;
    };
    (waiting, state.frames.get(&label).copied())
  };
  let Some(window) = app.get_webview_window(label.as_str()) else {
    return;
  };
  for then in waiting {
    then(Ok(window.clone()));
  }
  // Maximizing a hidden window would show it, so a window closed maximized is
  // maximized again once what opened it has shown it.
  if frame.is_some_and(|frame| frame.maximized) && window.is_visible().unwrap_or(false) {
    let _ = window.maximize();
  }
}

pub(super) fn failed(label: WindowLabel, error: String) {
  let waiting = match state().stages.remove(&label) {
    Some(Stage::Loading { waiting, .. }) => waiting,
    _ => Vec::new(),
  };
  eprintln!("Could not build the {} window: {error}", label.as_str());
  for then in waiting {
    then(Err(error.clone()));
  }
}

pub(super) fn reopened(label: WindowLabel, window: &WebviewWindow) {
  let waiting = match state().stages.insert(label, Stage::Ready) {
    Some(Stage::Closing(waiting)) => waiting,
    _ => Vec::new(),
  };
  for then in waiting {
    then(Ok(window.clone()));
  }
}
