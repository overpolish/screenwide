// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS presentation for the portable cursor lease.
//!
//! AppKit cursor changes only reach WindowServer reliably while Screenwide is
//! active. The first lease captures the foreground application; presentation
//! activates once the overlay is key, and the final release restores the app. Transfers retain
//! that foreground ownership and only replace the generation and cursor.

use std::{
  ffi::c_int,
  sync::{Mutex, OnceLock},
};

use tauri::AppHandle;

use super::{
  CursorLeaseError, CursorLeaseId, CursorLeaseState, CursorOwner, CursorPresentation,
  CursorTransition,
};
use crate::osc::protocol::CursorIcon;

#[derive(Clone, Default)]
struct Coordinator {
  leases: CursorLeaseState,
  owners: Vec<(CursorOwner, CursorLeaseId)>,
}

static COORDINATOR: OnceLock<Mutex<Coordinator>> = OnceLock::new();

unsafe extern "C" {
  fn screenwide_cursor_session_present(window: *mut std::ffi::c_void);
  fn screenwide_cursor_session_acquire(generation: u64, icon: u8) -> c_int;
  fn screenwide_cursor_session_update(generation: u64, icon: u8) -> c_int;
  fn screenwide_cursor_session_release(generation: u64) -> c_int;
  fn screenwide_cursor_session_transfer(from: u64, to: u64, icon: u8) -> c_int;
}

/// Called on the main thread after configuring the interactive overlay.
pub(crate) fn present_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  let native_window = window.ns_window()?;
  unsafe { screenwide_cursor_session_present(native_window) };
  Ok(())
}

/// Relinquish focus before closing an overlay, while its native window still
/// exists. Closing a key/main window otherwise promotes Settings/Export before
/// the asynchronous application restoration has finished.
pub(crate) fn prepare_window_close(window: &tauri::WebviewWindow) {
  if let Ok(raw_window) = window.ns_window() {
    let native_window: &objc2_app_kit::NSWindow = unsafe { &*raw_window.cast() };
    native_window.resignKeyWindow();
    native_window.resignMainWindow();
  }
}

fn coordinator() -> &'static Mutex<Coordinator> {
  COORDINATOR.get_or_init(|| Mutex::new(Coordinator::default()))
}

fn apply(transition: CursorTransition) -> Result<(), String> {
  let Some(presentation) = transition.presentation else {
    return Ok(());
  };
  let applied = unsafe {
    match presentation {
      CursorPresentation::Acquire(lease) => {
        screenwide_cursor_session_acquire(lease.id.generation(), lease.icon as u8)
      }
      CursorPresentation::Update(lease) => {
        screenwide_cursor_session_update(lease.id.generation(), lease.icon as u8)
      }
      CursorPresentation::Release(id) => screenwide_cursor_session_release(id.generation()),
      CursorPresentation::Transfer { from, to } => {
        screenwide_cursor_session_transfer(from.generation(), to.id.generation(), to.icon as u8)
      }
    }
  };
  if applied == 0 {
    Err(format!(
      "macOS rejected cursor presentation {presentation:?}"
    ))
  } else {
    Ok(())
  }
}

fn on_main_thread(
  app: &AppHandle,
  operation: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
  if objc2::MainThreadMarker::new().is_some() {
    return operation();
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = sender.send(operation());
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

fn lease_error(error: CursorLeaseError) -> String {
  format!("could not acquire cursor lease: {error:?}")
}

impl Coordinator {
  fn owner_id(&self, owner: CursorOwner) -> Option<CursorLeaseId> {
    self
      .owners
      .iter()
      .find_map(|(candidate, id)| (*candidate == owner).then_some(*id))
  }

  fn acquire(&mut self, owner: CursorOwner, icon: CursorIcon) -> Result<(), String> {
    if let Some(id) = self.owner_id(owner) {
      if self.leases.active().is_some_and(|lease| lease.id == id) {
        apply(self.leases.update(id, icon))?;
      }
      return Ok(());
    }

    let previous = self.clone();
    let (id, transition) = if let Some(active) = self.leases.active() {
      self
        .leases
        .transfer(active.id, owner, icon)
        .map_err(lease_error)?
    } else {
      self.leases.acquire(owner, icon).map_err(lease_error)?
    };
    self.owners.push((owner, id));
    if let Err(error) = apply(transition) {
      *self = previous;
      return Err(error);
    }
    Ok(())
  }

  fn release(&mut self, owner: CursorOwner) -> Result<(), String> {
    let Some(id) = self.owner_id(owner) else {
      return Ok(());
    };
    let previous = self.clone();
    let transition = if self.leases.active().is_some_and(|lease| lease.id == id) {
      if let Some(restorable) = self.leases.restorable() {
        self.leases.restore(id, restorable.id)
      } else {
        self.leases.release(id)
      }
    } else {
      self.leases.release(id)
    };
    self.owners.retain(|(candidate, _)| *candidate != owner);
    if let Err(error) = apply(transition) {
      *self = previous;
      return Err(error);
    }
    Ok(())
  }
}

fn acquire(app: &AppHandle, owner: CursorOwner, icon: CursorIcon) -> Result<(), String> {
  on_main_thread(app, move || {
    let mut coordinator = coordinator()
      .lock()
      .map_err(|_| "cursor coordinator lock was poisoned".to_owned())?;
    coordinator.acquire(owner, icon)
  })
}

fn release(app: &AppHandle, owner: CursorOwner) -> Result<(), String> {
  on_main_thread(app, move || {
    let mut coordinator = coordinator()
      .lock()
      .map_err(|_| "cursor coordinator lock was poisoned".to_owned())?;
    coordinator.release(owner)
  })
}

pub(crate) fn acquire_quick_screenshot(app: &AppHandle) -> Result<(), String> {
  acquire(app, CursorOwner::QuickScreenshot, CursorIcon::Crosshair)
}

pub(crate) fn release_quick_screenshot(app: &AppHandle) -> Result<(), String> {
  release(app, CursorOwner::QuickScreenshot)
}

pub(crate) fn acquire_ruler(app: &AppHandle) -> Result<(), String> {
  acquire(app, CursorOwner::Ruler, CursorIcon::Crosshair)
}

pub(crate) fn release_ruler(app: &AppHandle) -> Result<(), String> {
  release(app, CursorOwner::Ruler)
}

pub(crate) fn acquire_text_recognition(app: &AppHandle) -> Result<(), String> {
  acquire(app, CursorOwner::TextRecognition, CursorIcon::Crosshair)
}

pub(crate) fn release_text_recognition(app: &AppHandle) -> Result<(), String> {
  release(app, CursorOwner::TextRecognition)
}
