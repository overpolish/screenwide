// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use std::{
  collections::HashSet,
  path::PathBuf,
  sync::{
    atomic::{AtomicU64, Ordering},
    LazyLock, Mutex, Once,
  },
};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Preview {
  pub session_id: u64,
  pub desktop: String,
  pub selected: bool,
  pub origin: bool,
  pub index: usize,
  pub count: usize,
  pub phase: &'static str,
  pub icon_path: Option<PathBuf>,
}
static READY: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static CURRENT: LazyLock<Mutex<Vec<Preview>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static GENERATION: AtomicU64 = AtomicU64::new(0);
static POSITIONED: AtomicU64 = AtomicU64::new(0);
static INIT: Once = Once::new();
fn label(index: usize) -> String {
  format!("glide-space-{index}")
}

pub(super) fn initialize(app: &AppHandle) {
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
      if !ready.label.starts_with("glide-space-") {
        return;
      }
      {
        if let Ok(mut set) = READY.lock() {
          set.insert(ready.label.clone());
        }
      }
      let preview = CURRENT.lock().ok().and_then(|items| {
        items
          .iter()
          .find(|p| label(p.index) == ready.label)
          .cloned()
      });
      if let Some(preview) = preview {
        render(&handle, preview)
      }
    });
  });
}
pub(super) fn preload(app: &AppHandle) {
  initialize(app);
  let app = app.clone();
  let _ = std::thread::Builder::new()
    .name("glide-space-preload".to_owned())
    .spawn(move || {
      let monitor_count = app.available_monitors().map(|m| m.len()).unwrap_or(1);
      let desktop_count = if super::virtual_desktops::supported() {
        winvd::get_desktops()
          .map(|desktops| desktops.len())
          .unwrap_or(0)
      } else {
        0
      };
      let count = monitor_count.max(desktop_count);
      let main = app.clone();
      let _ = app.run_on_main_thread(move || {
        for index in 0..count {
          if let Err(error) = ensure(&main, index) {
            eprintln!("Could not preload Glide preview: {error}");
            break;
          }
        }
      });
    });
}
fn ensure(app: &AppHandle, index: usize) -> tauri::Result<tauri::WebviewWindow> {
  let name = label(index);
  if let Some(w) = app.get_webview_window(&name) {
    return Ok(w);
  }
  let mut config = app
    .config()
    .app
    .windows
    .iter()
    .find(|c| c.label == "glide")
    .ok_or(tauri::Error::WindowNotFound)?
    .clone();
  config.label = name;
  config.url = WebviewUrl::App("/glide-space".into());
  config.visible = false;
  config.focus = false;
  let w = WebviewWindowBuilder::from_config(app, &config)?.build()?;
  crate::windows::platform::initialize_glide_preview(&w)?;
  Ok(w)
}
pub(super) fn show_arranged(
  app: &AppHandle,
  previews: Vec<Preview>,
  x: f64,
  y: f64,
  offsets: &[(f64, f64)],
) {
  initialize(app);
  let offsets = offsets.to_vec();
  let generation = GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
  if let Ok(mut current) = CURRENT.lock() {
    *current = previews.clone();
  }
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if GENERATION.load(Ordering::Acquire) != generation {
      return;
    }
    let count = previews.len();
    for (name, window) in main.webview_windows() {
      let Some(index) = name
        .strip_prefix("glide-space-")
        .and_then(|value| value.parse::<usize>().ok())
      else {
        continue;
      };
      if index >= count {
        if let Err(error) = crate::windows::platform::hide(&window) {
          eprintln!("Could not hide surplus Glide preview {name}: {error}");
        }
      }
    }
    let monitor = main.available_monitors().ok().and_then(|items| {
      items.into_iter().find(|m| {
        let p = m.position();
        let s = m.size();
        x >= f64::from(p.x)
          && x < f64::from(p.x + s.width as i32)
          && y >= f64::from(p.y)
          && y < f64::from(p.y + s.height as i32)
      })
    });
    let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.);
    let width = (offsets.iter().map(|v| v.0).fold(0., f64::max) + 48.) * scale;
    let height = (offsets.iter().map(|v| v.1).fold(0., f64::max) + 32.) * scale;
    let mut left = x - width / 2.;
    let mut top = y - height / 2.;
    if let Some(m) = monitor {
      let w = m.work_area();
      left = left.clamp(
        f64::from(w.position.x),
        (f64::from(w.position.x + w.size.width as i32) - width).max(f64::from(w.position.x)),
      );
      top = top.clamp(
        f64::from(w.position.y),
        (f64::from(w.position.y + w.size.height as i32) - height).max(f64::from(w.position.y)),
      );
    }
    let mut positioned = true;
    for (i, (ox, oy)) in offsets.iter().enumerate() {
      let Ok(w) = ensure(&main, i) else {
        eprintln!("Could not create Glide monitor preview {i}");
        positioned = false;
        continue;
      };
      if let Err(error) = w.set_position(tauri::PhysicalPosition::new(
        (left + ox * scale).round() as i32,
        (top + oy * scale).round() as i32,
      )) {
        eprintln!("Could not position Glide monitor preview {i}: {error}");
        positioned = false;
        continue;
      }
      if let Err(error) = w.set_size(tauri::PhysicalSize::new(
        (48.0 * scale).round() as u32,
        (32.0 * scale).round() as u32,
      )) {
        eprintln!("Could not size Glide monitor preview {i}: {error}");
        positioned = false;
      }
    }
    if !positioned {
      return;
    }
    POSITIONED.store(generation, Ordering::Release);
    if let Some(first) = previews.into_iter().next() {
      if GENERATION.load(Ordering::Acquire) == generation {
        render(&main, first);
      }
    }
  });
}
fn render(app: &AppHandle, p: Preview) {
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if std::env::var_os("SCREENWIDE_GLIDE_TRACE").is_some() {
      eprintln!(
        "Glide preview render session={} index={}",
        p.session_id, p.index
      );
    }
    let Some(session) = CURRENT.lock().ok().and_then(|items| {
      let session = items
        .iter()
        .filter(|item| item.session_id == p.session_id)
        .cloned()
        .collect::<Vec<_>>();
      if session.is_empty()
        || GENERATION.load(Ordering::Acquire) == 0
        || POSITIONED.load(Ordering::Acquire) != GENERATION.load(Ordering::Acquire)
        || !READY.lock().is_ok_and(|ready| {
          session
            .iter()
            .all(|item| ready.contains(&label(item.index)))
        })
      {
        return None;
      }
      Some(session)
    }) else {
      return;
    };
    for latest in &session {
      if let Some(w) = main.get_webview_window(&label(latest.index)) {
        if let Err(error) = main.emit_to(w.label(), "glide://space-preview", latest) {
          eprintln!("Could not emit Glide preview: {error}");
          return;
        }
      }
    }
    for latest in session {
      if let Some(w) = main.get_webview_window(&label(latest.index)) {
        if let Err(error) = crate::windows::platform::show_glide(&w, 1.0, false) {
          eprintln!("Could not show Glide preview: {error}");
          return;
        }
      }
    }
  });
}
pub(super) fn hide(app: &AppHandle) {
  let generation = GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
  if let Ok(mut p) = CURRENT.lock() {
    p.clear();
  }
  let main = app.clone();
  let _ = app.run_on_main_thread(move || {
    if GENERATION.load(Ordering::Acquire) != generation {
      return;
    }
    for (name, w) in main.webview_windows() {
      if name.starts_with("glide-space-") {
        if let Err(error) = crate::windows::platform::hide(&w) {
          eprintln!("Could not hide Glide monitor preview {name}: {error}");
        }
      }
    }
  });
}

pub(super) fn set_icon(app: &AppHandle, session_id: u64, icon_path: Option<PathBuf>) {
  crate::glide::core::trace::input(
    "preview",
    format!("icon session={session_id} present={}", icon_path.is_some()),
  );
  let previews = CURRENT
    .lock()
    .ok()
    .map(|mut items| {
      let mut changed = Vec::new();
      for item in items
        .iter_mut()
        .filter(|item| item.session_id == session_id)
      {
        item.icon_path = icon_path.clone();
        changed.push(item.clone());
      }
      changed
    })
    .unwrap_or_default();
  if let Some(preview) = previews.into_iter().next() {
    render(app, preview);
  }
}
