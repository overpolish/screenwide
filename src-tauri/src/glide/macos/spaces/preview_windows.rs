// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each Space owns a normal-sized native Glide panel and its own material.
use serde::{Deserialize, Serialize};
use std::{
  collections::HashSet,
  path::PathBuf,
  sync::{LazyLock, Mutex, Once},
};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::glide::platform) struct Preview {
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
  ready: HashSet<String>,
  previews: Vec<Preview>,
  active: bool,
  fading: usize,
  after_fade: Option<Box<dyn FnOnce() + Send>>,
}
static POOL: LazyLock<Mutex<Pool>> = LazyLock::new(|| Mutex::new(Pool::default()));
static INIT: Once = Once::new();
fn label(index: usize) -> String {
  format!("glide-space-{index}")
}

pub(super) fn initialize(app: &AppHandle) {
  INIT.call_once(|| {
    let app = app.clone();
    let handle = app.clone();
    app.listen("glide://space-ready", move |event| {
      #[derive(Deserialize)]
      struct Ready {
        label: String,
      }
      let Ok(ready) = serde_json::from_str::<Ready>(event.payload()) else {
        return;
      };
      if !ready.label.starts_with("glide-space-") {
        return;
      }
      let preview = {
        let mut pool = POOL.lock().unwrap_or_else(|error| error.into_inner());
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

/// Begin loading the hidden webviews before the first gesture needs them.
pub(in crate::glide::platform) fn preload(app: &AppHandle) {
  initialize(app);
  let app = app.clone();
  let result = std::thread::Builder::new()
    .name("glide-space-preload".into())
    .spawn(move || {
      unsafe extern "C" {
        fn sw_glide_space_preview_count() -> usize;
      }
      // Read-only topology query; panels themselves are created on the UI thread.
      let count = unsafe { sw_glide_space_preview_count() }.max(
        app
          .available_monitors()
          .map(|monitors| monitors.len())
          .unwrap_or(0),
      );
      let main = app.clone();
      let _ = app.run_on_main_thread(move || {
        for index in 0..count {
          if let Err(error) = ensure_window(&main, index) {
            eprintln!("Could not preload Glide Space preview: {error}");
            break;
          }
        }
      });
    });
  if let Err(error) = result {
    eprintln!("Could not start Glide Space preload: {error}");
  }
}

fn ensure_window(app: &AppHandle, index: usize) -> tauri::Result<tauri::WebviewWindow> {
  let name = label(index);
  if let Some(window) = app.get_webview_window(&name) {
    return Ok(window);
  }
  let mut config = app
    .config()
    .app
    .windows
    .iter()
    .find(|config| config.label == "glide")
    .ok_or(tauri::Error::WindowNotFound)?
    .clone();
  config.label = name;
  config.url = WebviewUrl::App("/glide-space".into());
  config.visible = false;
  config.focus = false;
  let window = WebviewWindowBuilder::from_config(app, &config)?.build()?;
  crate::windows::platform::initialize_glide_preview(&window)?;
  Ok(window)
}

pub(super) fn show(app: &AppHandle, previews: Vec<Preview>, x: f64, y: f64) -> tauri::Result<()> {
  let offsets: Vec<_> = (0..previews.len())
    .map(|index| (index as f64 * 56.0, 0.0))
    .collect();
  show_arranged(app, previews, x, y, &offsets)
}

pub(in crate::glide::platform) fn show_arranged(
  app: &AppHandle,
  previews: Vec<Preview>,
  x: f64,
  y: f64,
  offsets: &[(f64, f64)],
) -> tauri::Result<()> {
  let count = previews.len();
  // Reuse the ordinary preview's anchor-monitor resolution, while it stays hidden.
  crate::windows::position_glide_preview(app, x, y)?;
  let normal = app
    .get_webview_window("glide")
    .ok_or(tauri::Error::WindowNotFound)?;
  let monitor = normal.current_monitor()?;
  let width = offsets
    .iter()
    .map(|(x, _)| *x)
    .reduce(f64::max)
    .unwrap_or(0.0)
    + 48.0;
  let height = offsets
    .iter()
    .map(|(_, y)| *y)
    .reduce(f64::max)
    .unwrap_or(0.0)
    + 32.0;
  let (mut left, mut top) = (x - width / 2.0, y - height / 2.0);
  if let Some(monitor) = monitor {
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let origin = work.position.to_logical::<f64>(scale);
    let size = work.size.to_logical::<f64>(scale);
    left = left.clamp(origin.x, (origin.x + size.width - width).max(origin.x));
    top = top.clamp(origin.y, (origin.y + size.height - height).max(origin.y));
  }
  for index in 0..count {
    let window = ensure_window(app, index)?;
    window.set_position(tauri::LogicalPosition::new(
      left + offsets[index].0,
      top + offsets[index].1,
    ))?;
  }
  let ready = {
    let mut pool = POOL.lock().unwrap_or_else(|error| error.into_inner());
    pool.previews = previews;
    pool.active = true;
    pool
      .previews
      .iter()
      .filter(|preview| pool.ready.contains(&label(preview.index)))
      .cloned()
      .collect::<Vec<_>>()
  };
  if let Some(preview) = ready.into_iter().next() {
    render(app, preview);
  }
  Ok(())
}

fn render(app: &AppHandle, preview: Preview) {
  let app = app.clone();
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    let Some(session) = POOL.lock().ok().and_then(|pool| {
      if !pool.active {
        return None;
      }
      let session = pool
        .previews
        .iter()
        .filter(|item| item.session_id == preview.session_id)
        .cloned()
        .collect::<Vec<_>>();
      if session.is_empty()
        || !session
          .iter()
          .all(|item| pool.ready.contains(&label(item.index)))
      {
        return None;
      }
      Some(session)
    }) else {
      return;
    };
    for preview in &session {
      if let Some(window) = main.get_webview_window(&label(preview.index)) {
        if main
          .emit_to(window.label(), "glide://space-preview", preview)
          .is_err()
        {
          return;
        }
      }
    }
    for preview in session {
      if let Some(window) = main.get_webview_window(&label(preview.index)) {
        if crate::windows::platform::show_glide(&window, 1.0, false).is_err() {
          return;
        }
      }
    }
  });
}

pub(in crate::glide::platform) fn dismiss(app: &AppHandle, cancelled: bool) {
  let windows: Vec<_> = app
    .webview_windows()
    .into_iter()
    .filter(|(label, _)| label.starts_with("glide-space-"))
    .map(|(_, window)| window)
    .collect();
  {
    let mut pool = POOL.lock().unwrap_or_else(|error| error.into_inner());
    // A committed carry may finish after its preview has already faded out.
    // Do not restart dismissal or replace the outstanding fade count.
    if !pool.active {
      return;
    }
    pool.active = false;
    pool.fading = if cancelled { 0 } else { windows.len() };
  }
  let _ = app.run_on_main_thread(move || {
    for window in windows {
      if cancelled {
        let _ = crate::windows::platform::hide(&window);
      } else if crate::windows::platform::fade_glide_preview(&window, Box::new(faded)).is_err() {
        let _ = crate::windows::platform::hide(&window);
        faded();
      }
    }
  });
}

fn faded() {
  let done = {
    let mut pool = POOL.lock().unwrap_or_else(|error| error.into_inner());
    pool.fading = pool.fading.saturating_sub(1);
    if pool.fading == 0 {
      pool.after_fade.take()
    } else {
      None
    }
  };
  if let Some(done) = done {
    done();
  }
}

/// A fresh gesture cannot reuse a panel until its previous fade has finished.
pub(in crate::glide::platform) fn after_dismissed(done: Box<dyn FnOnce() + Send>) {
  let mut pool = POOL.lock().unwrap_or_else(|error| error.into_inner());
  if pool.fading == 0 {
    drop(pool);
    done();
  } else {
    pool.after_fade = Some(done);
  }
}

pub(in crate::glide::platform) fn is_dismissing() -> bool {
  POOL.lock().is_ok_and(|pool| pool.fading > 0)
}
