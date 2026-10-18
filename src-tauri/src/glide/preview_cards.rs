// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cards the monitor picker and Spaces show: one hidden webview per card,
//! loaded ahead of the first gesture, arranged round the anchor inside its
//! monitor's work area, shown together once every one of them has loaded,
//! and faded out on a commit. Both platforms run this same pool; a platform
//! only says how many cards it might need and where its anchors are.

mod arrange;
mod dismiss;

use serde::Serialize;
use std::{
  collections::HashSet,
  path::PathBuf,
  sync::{
    atomic::{AtomicU64, Ordering},
    LazyLock, Mutex, MutexGuard, Once,
  },
};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};

pub(crate) use arrange::Anchor;
pub(crate) use dismiss::{after_dismissed, dismiss, is_dismissing};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Preview {
  pub session_id: u64,
  pub desktop: String,
  pub selected: bool,
  pub origin: bool,
  pub index: usize,
  pub count: usize,
  pub phase: &'static str,
  pub icon_path: Option<PathBuf>,
}

#[derive(Default)]
struct Pool {
  /// The cards whose webviews have loaded.
  ready: HashSet<String>,
  previews: Vec<Preview>,
  active: bool,
  /// Cards still fading out after a commit.
  fading: usize,
  after_fade: Option<Box<dyn FnOnce() + Send>>,
  /// The last icon looked up and the session it is for. The lookup can land
  /// between a session building its cards and showing them, so cards shown
  /// after it take the icon from here.
  icon: Option<(u64, Option<PathBuf>)>,
}

static POOL: LazyLock<Mutex<Pool>> = LazyLock::new(Mutex::default);
/// Bumped by every show and dismissal, so a placement that runs late on the
/// main thread drops its result rather than undo a newer one.
static GENERATION: AtomicU64 = AtomicU64::new(0);
/// The generation whose cards are in place, so nothing is shown before then.
static POSITIONED: AtomicU64 = AtomicU64::new(0);
static INIT: Once = Once::new();

const PREFIX: &str = "glide-space-";

fn label(index: usize) -> String {
  format!("{PREFIX}{index}")
}

fn pool() -> MutexGuard<'static, Pool> {
  POOL.lock().unwrap_or_else(|error| error.into_inner())
}

fn initialize(app: &AppHandle) {
  INIT.call_once(|| {
    let handle = app.clone();
    app.listen("glide://space-ready", move |event| {
      #[derive(serde::Deserialize)]
      struct Ready {
        label: String,
      }
      let Ok(ready) = serde_json::from_str::<Ready>(event.payload()) else {
        return;
      };
      if !ready.label.starts_with(PREFIX) {
        return;
      }
      let preview = {
        let mut pool = pool();
        pool.ready.insert(ready.label.clone());
        pool
          .active
          .then(|| {
            pool
              .previews
              .iter()
              .find(|preview| label(preview.index) == ready.label)
              .cloned()
          })
          .flatten()
      };
      if let Some(preview) = preview {
        render(&handle, preview);
      }
    });
  });
}

/// Loads `count` hidden cards before the first gesture needs them.
pub(crate) fn preload(app: &AppHandle, count: usize) {
  initialize(app);
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    for index in 0..count {
      if let Err(error) = ensure(&main, index) {
        eprintln!("Could not preload Glide preview: {error}");
        break;
      }
    }
  });
}

fn ensure(app: &AppHandle, index: usize) -> tauri::Result<tauri::WebviewWindow> {
  let name = label(index);
  if let Some(window) = app.get_webview_window(&name) {
    return Ok(window);
  }
  let mut config = crate::app_windows::WindowLabel::Glide
    .config(app)
    .ok_or(tauri::Error::WindowNotFound)?;
  config.label = name;
  config.url = WebviewUrl::App("/glide-space".into());
  config.visible = false;
  config.focus = false;
  let window = WebviewWindowBuilder::from_config(app, &config)?
    .initialization_script(screenwide_accent::initialization_script())
    .build()?;
  crate::app_windows::platform::initialize_glide_preview(&window)?;
  Ok(window)
}

/// Shows `previews` round `anchor`, each card at its offset in logical
/// points from the arrangement's corner.
pub(crate) fn show_arranged(
  app: &AppHandle,
  mut previews: Vec<Preview>,
  anchor: Anchor,
  offsets: &[(f64, f64)],
) {
  initialize(app);
  let offsets = offsets.to_vec();
  let generation = GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
  {
    let mut pool = pool();
    if let Some((session_id, icon_path)) = &pool.icon {
      for preview in previews
        .iter_mut()
        .filter(|preview| preview.session_id == *session_id && preview.icon_path.is_none())
      {
        preview.icon_path = icon_path.clone();
      }
    }
    pool.previews = previews.clone();
    pool.active = true;
  }
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if GENERATION.load(Ordering::Acquire) != generation {
      return;
    }
    hide_surplus(&main, offsets.len());
    let placements = arrange::place(&main, anchor, &offsets);
    let mut positioned = true;
    for (index, placement) in placements.into_iter().enumerate() {
      let placed = ensure(&main, index).and_then(|window| {
        window.set_position(placement.position)?;
        window.set_size(placement.size)
      });
      if let Err(error) = placed {
        eprintln!("Could not place Glide preview {index}: {error}");
        positioned = false;
      }
    }
    if !positioned {
      return;
    }
    POSITIONED.store(generation, Ordering::Release);
    if let Some(first) = previews.into_iter().next() {
      render(&main, first);
    }
  });
}

/// Hides the cards a smaller arrangement leaves over.
fn hide_surplus(app: &AppHandle, count: usize) {
  for (name, window) in app.webview_windows() {
    let Some(index) = name
      .strip_prefix(PREFIX)
      .and_then(|value| value.parse::<usize>().ok())
    else {
      continue;
    };
    if index >= count {
      if let Err(error) = crate::app_windows::platform::hide(&window) {
        eprintln!("Could not hide surplus Glide preview {name}: {error}");
      }
    }
  }
}

/// Sends every card of `preview`'s session its state and shows them all,
/// once they are placed and every one of them has loaded.
fn render(app: &AppHandle, preview: Preview) {
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    let session = {
      let pool = pool();
      let session = pool
        .previews
        .iter()
        .filter(|item| item.session_id == preview.session_id)
        .cloned()
        .collect::<Vec<_>>();
      let generation = GENERATION.load(Ordering::Acquire);
      let ready = pool.active
        && !session.is_empty()
        && POSITIONED.load(Ordering::Acquire) == generation
        && session
          .iter()
          .all(|item| pool.ready.contains(&label(item.index)));
      if !ready {
        return;
      }
      session
    };
    for item in &session {
      if let Some(window) = main.get_webview_window(&label(item.index)) {
        if let Err(error) = main.emit_to(window.label(), "glide://space-preview", item) {
          eprintln!("Could not emit Glide preview: {error}");
        }
      }
    }
    for item in session {
      if let Some(window) = main.get_webview_window(&label(item.index)) {
        if let Err(error) = crate::app_windows::platform::show_glide(&window, 1.0, false) {
          eprintln!("Could not show Glide preview: {error}");
        }
      }
    }
  });
}

/// Updates the process icon every card of `session_id` shows.
pub(crate) fn set_icon(app: &AppHandle, session_id: u64, icon_path: Option<PathBuf>) {
  let changed = {
    let mut pool = pool();
    pool.icon = Some((session_id, icon_path.clone()));
    let mut changed = Vec::new();
    for item in pool
      .previews
      .iter_mut()
      .filter(|item| item.session_id == session_id)
    {
      item.icon_path = icon_path.clone();
      changed.push(item.clone());
    }
    changed
  };
  if let Some(preview) = changed.into_iter().next() {
    render(app, preview);
  }
}
