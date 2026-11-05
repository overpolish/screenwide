// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A reference on screen, the same on every platform: its picture and chrome
//! drawn through the shared OSC renderer, the window placed round the
//! picture (`fit`), the pointer moving and resizing it, and the glide back
//! to a place and size. The platform gives it a window, a view and the
//! pointer through [`ReferenceHost`] (`references::macos`,
//! `references::windows`). Everything here runs on the main thread, where
//! the windows are.

mod actions;
mod glide;
mod host;
mod input;
mod place;

use std::cell::RefCell;
use std::collections::HashMap;

use screenwide_gpu::surface::Surface;
use screenwide_osc::geometry::{Rect, Size};
use screenwide_workspace::editing::ChromeRenderer;
use tauri::{AppHandle, Position};

use super::fit::Fit;
use super::stage::Stage;

pub(super) use actions::{fade_out, fit_island, reset_size, set_opacity, to_origin};
pub(crate) use host::{Cursor, ReferenceHost};
pub(super) use input::{down, dragged, hover, takes, up};
#[cfg(target_os = "windows")]
pub(super) use place::rescale;

#[cfg(target_os = "macos")]
pub(super) type PlatformHost = super::macos::MacHost;
#[cfg(target_os = "windows")]
pub(super) type PlatformHost = super::windows::WindowsHost;

/// Tells a reference's page where its island now stands.
const LAYOUT_EVENT: &str = "reference-layout";

struct Native {
  app: AppHandle,
  label: String,
  host: PlatformHost,
  picture: ChromeRenderer,
  chrome: ChromeRenderer,
  stage: Stage,
  fit: Fit,
  /// The latest glide back to a place and size (`glide`), which any older
  /// one gives way to.
  glide: u64,
}

thread_local! {
  static NATIVES: RefCell<HashMap<String, Native>> = RefCell::new(HashMap::new());
}

/// Runs `work` on the reference `label`. None where it is gone, or already
/// being worked on further up this thread's stack, as the platform can call
/// back into a view while its window is being moved.
fn with<R>(label: &str, work: impl FnOnce(&mut Native) -> R) -> Option<R> {
  NATIVES.with(|natives| {
    let mut natives = natives.try_borrow_mut().ok()?;
    natives.get_mut(label).map(work)
  })
}

/// Shows `texture` at `size` in the reference `label`'s view, drawn on
/// `picture` and `chrome`, with its corner at `origin`, or centred on the
/// main display.
pub(super) fn attach(
  app: &AppHandle,
  label: &str,
  host: PlatformHost,
  (picture, chrome): (Surface, Surface),
  texture: &wgpu::Texture,
  size: Size,
  origin: Option<Position>,
) -> Result<(), String> {
  let gpu = screenwide_gpu::shared()?;
  let snapshot = texture.create_view(&Default::default());
  let origin = origin.map_or_else(|| host.centred(size), |at| host.desktop_point(at));
  let mut native = Native {
    app: app.clone(),
    label: label.to_owned(),
    host,
    picture: ChromeRenderer::with_snapshot(gpu, picture, &snapshot),
    chrome: ChromeRenderer::with_snapshot(gpu, chrome, &snapshot),
    stage: Stage::new(size),
    fit: Fit::default(),
    glide: 0,
  };
  place::place(&mut native, Rect { origin, size });
  NATIVES.with(|natives| natives.borrow_mut().insert(label.to_owned(), native));
  Ok(())
}

/// Lets go of the reference `label`'s view, its drawing and its hit area.
pub(super) fn detach(label: &str) {
  let native = NATIVES.with(|natives| natives.borrow_mut().remove(label));
  if let Some(native) = native {
    native.host.detach();
    crate::app_windows::options::hit_area::set_hit_area(native.app, native.label, None);
  }
}

/// Runs `work` on the main thread, where the windows are, and waits for it.
pub(super) fn on_main<R: Send + 'static>(
  app: &AppHandle,
  work: impl FnOnce() -> R + Send + 'static,
) -> Result<R, String> {
  // Rust names the process's first thread, which Tauri's event loop runs
  // on, `main`.
  if std::thread::current().name() == Some("main") {
    return Ok(work());
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = sender.send(work());
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())
}
