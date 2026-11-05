// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A reference on macOS: a view over its webview carrying two Metal layers,
//! the picture's and the chrome's, drawn through the shared OSC renderer, and
//! the window placed round the picture (`fit`). Everything here runs on the
//! main thread, where the views are. Desktop lengths are in points from the
//! primary display's top left, as Tauri gives a window's logical position.

mod animate;
mod desktop;
mod draw;
mod input;
mod place;
mod view;

use std::cell::RefCell;
use std::collections::HashMap;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSMainMenuWindowLevel, NSView, NSWindow, NSWindowOrderingMode};
use objc2_foundation::NSRect;
use objc2_quartz_core::{CAMetalLayer, CATransaction};
use screenwide_osc::geometry::{Point, Rect, Size};
use screenwide_workspace::editing::ChromeRenderer;
use tauri::{AppHandle, Manager, WebviewWindow};

use super::commands::ReferenceLayout;
use super::fit::Fit;
use super::stage::Stage;
use desktop::{centred, pointer, screen_at};
use draw::draw;
use place::{layout, picture_on_desktop, place};
use view::{metal_layer, surface_on, ReferenceView};

/// Tells a reference's page where its island now starts.
pub(super) const LAYOUT_EVENT: &str = "reference-layout";
/// How long a reference takes to fade as it closes, in seconds: the
/// `travel` timing the island tucks away over (`src/lib/motion.ts`).
const FADE_SECONDS: f64 = 0.22;

struct Native {
  app: AppHandle,
  label: String,
  window: Retained<NSWindow>,
  host: Retained<NSView>,
  view: Retained<ReferenceView>,
  picture_layer: Retained<CAMetalLayer>,
  chrome_layer: Retained<CAMetalLayer>,
  picture: ChromeRenderer,
  chrome: ChromeRenderer,
  stage: Stage,
  fit: Fit,
  /// The latest glide back to a place and size (`animate`), which any older
  /// one gives way to.
  glide: u64,
}

thread_local! {
  static NATIVES: RefCell<HashMap<String, Native>> = RefCell::new(HashMap::new());
}

/// Runs `work` on the reference `label`. None where it is gone, or already
/// being worked on further up this thread's stack, as AppKit can call back
/// into a view while its window is being moved.
fn with<R>(label: &str, work: impl FnOnce(&mut Native) -> R) -> Option<R> {
  NATIVES.with(|natives| {
    let mut natives = natives.try_borrow_mut().ok()?;
    natives.get_mut(label).map(work)
  })
}

/// Gives the reference's `window` its view and layers, showing `texture`
/// at `size` with its corner at `origin`, or centred on the main display.
pub(super) fn attach(
  mtm: MainThreadMarker,
  window: &WebviewWindow,
  texture: &wgpu::Texture,
  size: Size,
  origin: Option<Point>,
) -> Result<(), String> {
  let gpu = screenwide_gpu::shared()?;
  let ns_window = window.ns_window().map_err(|error| error.to_string())?;
  let host = window.ns_view().map_err(|error| error.to_string())?;
  // Safety: Tauri hands out the live window and its content view.
  let (ns_window, host) = unsafe {
    (
      Retained::retain(ns_window.cast::<NSWindow>()),
      Retained::retain(host.cast::<NSView>()),
    )
  };
  let (Some(ns_window), Some(host)) = (ns_window, host) else {
    return Err("The reference's window has no view".to_owned());
  };
  // At the menu bar's level AppKit no longer keeps the window's top under
  // the menu bar, so a picture of the top of the screen can lie over it,
  // and the room for shadows above the picture never holds it down. Below
  // that level every frame set is held under it.
  ns_window.setLevel(NSMainMenuWindowLevel);
  let scale = ns_window.backingScaleFactor();
  let view = ReferenceView::new(mtm, window.label(), NSRect::ZERO);
  view.setWantsLayer(true);
  let root = view.layer().ok_or("The reference's view has no layer")?;
  let picture_layer = metal_layer(scale);
  let chrome_layer = metal_layer(scale);
  root.addSublayer(&picture_layer);
  root.addSublayer(&chrome_layer);
  let page = crate::app_windows::platform::webview_view(&host);
  host.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Above, page.as_deref());
  // Safety: on the main thread, on layers the view keeps for as long as the
  // surfaces live.
  let (picture_surface, chrome_surface) = unsafe {
    (
      surface_on(gpu, &picture_layer)?,
      surface_on(gpu, &chrome_layer)?,
    )
  };
  let snapshot = texture.create_view(&Default::default());
  let origin = origin.unwrap_or_else(|| centred(mtm, size));
  let mut native = Native {
    app: window.app_handle().clone(),
    label: window.label().to_owned(),
    window: ns_window,
    host,
    view,
    picture_layer,
    chrome_layer,
    picture: ChromeRenderer::with_snapshot(gpu, picture_surface, &snapshot),
    chrome: ChromeRenderer::with_snapshot(gpu, chrome_surface, &snapshot),
    stage: Stage::new(size),
    fit: Fit {
      window: Rect::default(),
      picture: Rect::default(),
      island: Rect::default(),
      island_above: false,
    },
    glide: 0,
  };
  place(mtm, &mut native, Rect { origin, size });
  NATIVES.with(|natives| {
    natives
      .borrow_mut()
      .insert(window.label().to_owned(), native)
  });
  Ok(())
}

/// Runs `work` on the main thread, where the views are, and waits for it.
pub(super) fn on_main<R: Send + 'static>(
  app: &AppHandle,
  work: impl FnOnce(MainThreadMarker) -> R + Send + 'static,
) -> Result<R, String> {
  if let Some(mtm) = MainThreadMarker::new() {
    return Ok(work(mtm));
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      if let Some(mtm) = MainThreadMarker::new() {
        let _ = sender.send(work(mtm));
      }
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())
}

/// Lets go of the reference `label`'s view and layers.
pub(super) fn detach(label: &str) {
  let native = NATIVES.with(|natives| natives.borrow_mut().remove(label));
  if let Some(native) = native {
    native.view.removeFromSuperview();
    crate::app_windows::options::hit_area::set_hit_area(native.app, native.label, None);
  }
}

/// Fades the picture and its shadow out as the island tucks away, before
/// the window closes.
pub(super) fn fade_out(label: &str) {
  with(label, |native| {
    CATransaction::begin();
    CATransaction::setAnimationDuration(FADE_SECONDS);
    native.picture_layer.setOpacity(0.0);
    native.chrome_layer.setOpacity(0.0);
    CATransaction::commit();
  });
}

/// Fits the window round an island of `island` under the picture. Answers
/// where the island stands.
pub(super) fn fit_island(
  mtm: MainThreadMarker,
  label: &str,
  island: Size,
) -> Option<ReferenceLayout> {
  with(label, |native| {
    native.stage.island = island;
    let picture = picture_on_desktop(mtm, native);
    place(mtm, native, picture);
    layout(native)
  })
}

pub(super) fn set_opacity(label: &str, opacity: f64) {
  with(label, |native| {
    native.picture_layer.setOpacity(opacity as f32)
  });
}

/// Glides the picture back to `origin` at its true size.
pub(super) fn to_origin(mtm: MainThreadMarker, label: &str, origin: Point) {
  let Some(size) = with(label, |native| native.stage.true_size) else {
    return;
  };
  animate::glide_to(mtm, label, Rect { origin, size });
}

/// Glides the picture back to its true size, its middle where it is.
pub(super) fn reset_size(mtm: MainThreadMarker, label: &str) {
  let target = with(label, |native| {
    let shown = picture_on_desktop(mtm, native);
    let size = native.stage.true_size;
    Rect {
      origin: Point {
        x: (shown.origin.x + (shown.size.width - size.width) / 2.0).round(),
        y: (shown.origin.y + (shown.size.height - size.height) / 2.0).round(),
      },
      size,
    }
  });
  if let Some(target) = target {
    animate::glide_to(mtm, label, target);
  }
}
