// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A capture tool's shortcut pressed while one of Screenwide's menus is open,
//! the tray's or one in the menu bar. The menu belongs to this process, so
//! the first click anywhere else closes it, and no overlay can draw over it
//! and leave it open. Instead the screen is frozen with the menu in it, the
//! picture stands in front of the desktop, the menu closes, and the tool
//! works over the picture: Quick Screenshot selects from it and takes it,
//! the Ruler measures it, Annotate draws over it. The picture goes when that
//! tool ends.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use screenwide_imaging::CapturedImage;
use tauri::{AppHandle, PhysicalPosition, PhysicalSize};

use crate::shortcuts::ShortcutAction;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  pub(super) fn observe(_app: &tauri::AppHandle) {}
  pub(super) fn menu_open() -> bool {
    false
  }
  pub(super) fn close_menu() {}
  pub(super) fn show(_pictures: &[super::Picture]) {}
  pub(super) fn hide() {}
}

/// How long a tool is given to start over the picture. One that has not by
/// then failed to, and the picture would otherwise stay in front of the
/// desktop with nothing to end it.
const START_GRACE: Duration = Duration::from_secs(5);

/// The tool the picture was frozen for, which takes it away as it ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
  Screenshot,
  Annotate,
  Ruler,
}

/// One display frozen, and where that display is.
struct Picture {
  image: CapturedImage,
  position: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
  scale: f64,
}

/// The tool the picture up is for, with the freeze it came from, so a check
/// for a tool that never started leaves a later freeze alone.
static SHOWN: Mutex<Option<(Owner, u64)>> = Mutex::new(None);
static NEXT_FREEZE: AtomicU64 = AtomicU64::new(1);

fn shown() -> std::sync::MutexGuard<'static, Option<(Owner, u64)>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Starts watching for menus opening and closing. On the main thread.
pub(crate) fn initialize(app: &AppHandle) {
  platform::observe(app);
}

/// Whether a frozen picture stands in front of the desktop, which a
/// screenshot then takes with Screenwide's windows in.
pub(crate) fn is_up() -> bool {
  shown().is_some()
}

/// The tool `action` starts, if it would start one now.
fn owner_for(app: &AppHandle, action: ShortcutAction) -> Option<Owner> {
  match action {
    ShortcutAction::TakeScreenshot | ShortcutAction::TakeReferenceScreenshot => {
      Some(Owner::Screenshot)
    }
    // Pressed again while open, these close their tool instead.
    ShortcutAction::AnnotateOverlay if !crate::annotate::is_active(app) => Some(Owner::Annotate),
    ShortcutAction::RulerOverlay if !crate::ruler::is_active(app) => Some(Owner::Ruler),
    _ => None,
  }
}

fn owner_active(app: &AppHandle, owner: Owner) -> bool {
  match owner {
    Owner::Screenshot => crate::app_windows::region::is_screenshot_region_session(),
    Owner::Annotate => crate::annotate::is_active(app),
    Owner::Ruler => crate::ruler::is_active(app),
  }
}

/// Takes `action` over when a menu is open and it starts a capture tool:
/// the screen is frozen, the menu closed, and the action carried out over
/// the picture. Answers whether it did.
pub(crate) fn intercept(app: &AppHandle, action: ShortcutAction) -> bool {
  let Some(owner) = owner_for(app, action) else {
    return false;
  };
  if !platform::menu_open() {
    return false;
  }
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let pictures = match freeze(&app).await {
      Ok(pictures) => pictures,
      Err(error) => {
        // The tool still starts, over the live screen, as it would have.
        eprintln!("Could not freeze the screen with the menu open: {error}");
        Vec::new()
      }
    };
    let main = app.clone();
    let _ = app.run_on_main_thread(move || {
      let freeze = (!pictures.is_empty()).then(|| {
        let freeze = NEXT_FREEZE.fetch_add(1, Ordering::Relaxed);
        hide_now();
        platform::show(&pictures);
        *shown() = Some((owner, freeze));
        freeze
      });
      platform::close_menu();
      crate::shortcuts::run_over_frozen_menu(&main, action);
      if let Some(freeze) = freeze {
        release_if_never_started(&main, owner, freeze);
      }
    });
  });
  true
}

async fn freeze(app: &AppHandle) -> Result<Vec<Picture>, String> {
  let monitors = crate::capture_overlays::monitor_layout(app)?;
  let ids = monitors.iter().map(|(id, _, _)| *id).collect::<Vec<_>>();
  let stills =
    crate::screenshots::freeze_monitors(&ids, crate::screenshots::FrozenDesktop::WithOwnWindows)
      .await;
  monitors
    .into_iter()
    .zip(stills)
    .map(|((_, scale, monitor), still)| {
      still.map(|image| Picture {
        image,
        position: *monitor.position(),
        size: *monitor.size(),
        scale,
      })
    })
    .collect()
}

/// Takes the picture away once its tool has had time to start, if it has
/// not: nothing else would.
fn release_if_never_started(app: &AppHandle, owner: Owner, freeze: u64) {
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    tokio::time::sleep(START_GRACE).await;
    if *shown() != Some((owner, freeze)) || owner_active(&app, owner) {
      return;
    }
    release(&app, owner);
  });
}

/// The tool the picture was frozen for ended; any other leaves it.
pub(crate) fn release(app: &AppHandle, owner: Owner) {
  if !shown().is_some_and(|(shown, _)| shown == owner) {
    return;
  }
  let _ = app.run_on_main_thread(move || {
    if shown().is_some_and(|(shown, _)| shown == owner) {
      hide_now();
    }
  });
}

/// On the main thread.
fn hide_now() {
  platform::hide();
  *shown() = None;
}
