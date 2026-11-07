// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The windows built when first wanted rather than at launch: both editors
//! and their tool panels, Projects, Settings, the region overlay, the shared
//! menu window and Glide. Each is built from its `tauri.conf.json` entry and
//! destroyed when it closes, so a Screenwide with nothing open holds only the
//! island, which must appear the moment a shortcut asks, and Glide while it is
//! on and licensed (`crate::glide::window`). The region overlay and the menu
//! window are never closed by what uses them, so they go once they have stayed
//! hidden for a while (`release`).
//!
//! A new window is shown only once its page says it has rendered
//! (`window_ready`), so it never appears empty; what asked for it waits until
//! then. Building runs off the main thread: WebView2 cannot create a webview
//! from inside a command the main thread is answering.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use super::WindowLabel;

mod creation;
mod readiness;
mod release;
use creation::build_in_background;
use readiness::{failed, ready, reopened};

const ON_DEMAND: &[WindowLabel] = &[
  WindowLabel::EditorRecording,
  WindowLabel::EditorScreenshot,
  WindowLabel::ToolPanelRecording,
  WindowLabel::ToolPanelScreenshot,
  WindowLabel::Projects,
  WindowLabel::Settings,
  WindowLabel::RegionSelector,
  WindowLabel::StandaloneListbox,
  WindowLabel::Glide,
];

/// Whether `label`'s window is built when wanted, by [`open`], rather than at
/// launch.
pub(crate) fn is_on_demand(label: WindowLabel) -> bool {
  ON_DEMAND.contains(&label)
}

/// The windows whose page carries the title bar. On macOS they are configured
/// with a native overlay title bar, so the real traffic lights sit over the
/// page; on Windows the page draws its own caption buttons and wants no native
/// frame. `decorations` is a single configuration key for both platforms, so
/// the Windows half is undone as each is built. These are also the windows a
/// person moves and sizes, so each opens again where it was last closed.
const TITLE_BAR_IN_PAGE: &[WindowLabel] = &[
  WindowLabel::EditorRecording,
  WindowLabel::EditorScreenshot,
  WindowLabel::Projects,
  WindowLabel::Settings,
];

/// How long a new window's page may take to say it has rendered before the
/// window is shown anyway: a page that never says so must not leave its window
/// out of reach.
const READY_TIMEOUT: Duration = Duration::from_secs(5);

type Then = Box<dyn FnOnce(Result<WebviewWindow, String>) + Send>;

enum Stage {
  /// Being built, or its page still loading. `build` names this build, so a
  /// timeout left over from an earlier one does not mark this one ready.
  Loading {
    build: u64,
    waiting: Vec<Then>,
  },
  Ready,
  /// Being put away and destroyed. Asked for meanwhile, it is built again
  /// once gone.
  Closing(Vec<Then>),
}

/// Where a window stood when it was last closed.
#[derive(Clone, Copy)]
struct Frame {
  position: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
  maximized: bool,
}

#[derive(Default)]
struct OnDemand {
  stages: HashMap<WindowLabel, Stage>,
  frames: HashMap<WindowLabel, Frame>,
}

static ON_DEMAND_WINDOWS: LazyLock<Mutex<OnDemand>> = LazyLock::new(Default::default);
static BUILDS: AtomicU64 = AtomicU64::new(0);

fn state() -> MutexGuard<'static, OnDemand> {
  ON_DEMAND_WINDOWS
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
}

/// Hands `then` the window for `label` once it is ready to be shown,
/// building it first when it is not open.
pub(crate) fn open(
  app: &AppHandle,
  label: WindowLabel,
  then: impl FnOnce(Result<WebviewWindow, String>) + Send + 'static,
) {
  open_at(app, label, None, then);
}

/// [`open`], with `query` added to the page's address if the window has to
/// be built, so the page can start somewhere other than where it usually does.
pub(crate) fn open_at(
  app: &AppHandle,
  label: WindowLabel,
  query: Option<&'static str>,
  then: impl FnOnce(Result<WebviewWindow, String>) + Send + 'static,
) {
  let then: Then = Box::new(then);
  let mut state = state();
  let ready = match state.stages.get_mut(&label) {
    Some(Stage::Loading { waiting, .. } | Stage::Closing(waiting)) => {
      waiting.push(then);
      return;
    }
    Some(Stage::Ready) => true,
    None => false,
  };
  if ready {
    if let Some(window) = app.get_webview_window(label.as_str()) {
      drop(state);
      then(Ok(window));
      return;
    }
  }
  let build = BUILDS.fetch_add(1, Ordering::Relaxed);
  state.stages.insert(
    label,
    Stage::Loading {
      build,
      waiting: vec![then],
    },
  );
  drop(state);
  build_in_background(app.clone(), label, query, build);
}

/// Builds the window for `label` without showing it, so it is ready by the
/// time it is wanted.
pub(crate) fn prepare(app: &AppHandle, label: WindowLabel) {
  open(app, label, |_| {});
}

/// [`open`], awaited: the window for `label` once it is ready, built first
/// when it is not open.
pub(crate) async fn opened(app: &AppHandle, label: WindowLabel) -> Result<WebviewWindow, String> {
  let (sender, receiver) = std::sync::mpsc::channel();
  open(app, label, move |window| {
    let _ = sender.send(window);
  });
  tauri::async_runtime::spawn_blocking(move || receiver.recv())
    .await
    .map_err(|error| error.to_string())?
    .map_err(|_| {
      format!(
        "The {} window was closed before it was ready",
        label.as_str()
      )
    })?
}

/// Puts the window for `label` away and destroys it, its tool panel with it.
/// Whatever was still waiting for it to be built is dropped: closing came
/// after.
pub(crate) fn close(app: &AppHandle, label: WindowLabel) {
  if let Some(panel) = tool_panel_of(label) {
    close(app, panel);
  }
  let Some(window) = app.get_webview_window(label.as_str()) else {
    return;
  };
  {
    let mut state = state();
    if matches!(state.stages.get(&label), Some(Stage::Closing(_))) {
      return;
    }
    if TITLE_BAR_IN_PAGE.contains(&label) {
      if let Some(frame) = frame_of(&window) {
        state.frames.insert(label, frame);
      }
    }
    state.stages.insert(label, Stage::Closing(Vec::new()));
  }
  let dismissed = super::dismissal::dismiss(
    &window,
    Some(Box::new(move |window: &WebviewWindow, hidden: bool| {
      if hidden {
        let _ = window.destroy();
      } else {
        // Shown again before it was put away: it stays.
        reopened(label, window);
      }
    })),
  );
  if dismissed.is_err() {
    let _ = window.destroy();
  }
}

/// A new page saying it has rendered, so its window can be shown.
#[tauri::command]
pub fn window_ready(app: AppHandle, window: WebviewWindow) {
  if let Some(label) = ON_DEMAND
    .iter()
    .copied()
    .find(|label| label.as_str() == window.label())
  {
    ready(&app, label, None);
  }
}

fn tool_panel_of(label: WindowLabel) -> Option<WindowLabel> {
  match label {
    WindowLabel::EditorRecording => Some(WindowLabel::ToolPanelRecording),
    WindowLabel::EditorScreenshot => Some(WindowLabel::ToolPanelScreenshot),
    _ => None,
  }
}

fn frame_of(window: &WebviewWindow) -> Option<Frame> {
  if !window.is_visible().ok()? || window.is_minimized().ok()? {
    return None;
  }
  Some(Frame {
    position: window.outer_position().ok()?,
    size: window.inner_size().ok()?,
    maximized: window.is_maximized().ok()?,
  })
}

fn destroyed(app: &AppHandle, label: WindowLabel) {
  #[cfg(target_os = "macos")]
  super::platform::forget_panel(app, label);
  let mut state = state();
  let waiting = match state.stages.remove(&label) {
    Some(Stage::Closing(waiting)) => waiting,
    _ => Vec::new(),
  };
  if waiting.is_empty() {
    return;
  }
  let build = BUILDS.fetch_add(1, Ordering::Relaxed);
  state
    .stages
    .insert(label, Stage::Loading { build, waiting });
  drop(state);
  build_in_background(app.clone(), label, None, build);
}
