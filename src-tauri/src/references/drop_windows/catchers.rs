// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A catcher over each display while a drag is under way with Alt held: a
//! topmost popup the pointer sees almost nothing of, kept out of captures,
//! registered as an OLE drop target. OLE calls it on the thread that
//! registered it, the main one; whatever a drop needs is read during the
//! call, since the dragged data is only there for its length, and anything
//! that takes the catchers away waits until the call has returned.

use std::cell::Cell;
use std::sync::OnceLock;

use tauri::{PhysicalPosition, Position};
use windows::core::{implement, w, Ref, Result as ComResult, PCWSTR};
use windows::Win32::{
  Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, POINTL, WPARAM},
  System::{
    Com::IDataObject,
    Ole::{
      IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, DROPEFFECT, DROPEFFECT_COPY,
      DROPEFFECT_NONE,
    },
    SystemServices::MODIFIERKEYS_FLAGS,
  },
  UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, SetLayeredWindowAttributes, ShowWindow, HMENU,
    LWA_ALPHA, MA_NOACTIVATE, SW_SHOWNOACTIVATE, WM_MOUSEACTIVATE, WNDCLASS_STYLES, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
  },
};

use super::super::drop::{is_picture_address, Dropped};
use super::super::drop_watch;
use super::super::windows::desktop::scale_at;
use crate::app_windows::overlay_surface;
use crate::editor::images::dragged_windows;
use crate::editor::images::dropped::DroppedPicture;

const CLASS: PCWSTR = w!("ScreenwideReferenceCatcher");
/// Not quite clear: a window the pointer sees nothing of is passed over for
/// drops as it is for clicks.
const ALPHA: u8 = 1;

static CLASS_ATOM: OnceLock<u16> = OnceLock::new();

pub(in crate::references) struct Catchers(Vec<HWND>);

impl Catchers {
  pub(in crate::references) fn put_up() -> Self {
    let Some(app) = super::APP.get() else {
      return Self(Vec::new());
    };
    CLASS_ATOM.get_or_init(|| {
      overlay_surface::register_class(CLASS, Some(window_proc), WNDCLASS_STYLES(0))
    });
    let displays = app.available_monitors().unwrap_or_default();
    Self(
      displays
        .iter()
        .filter_map(|display| {
          let (at, size) = (display.position(), display.size());
          catcher(at.x, at.y, size.width as i32, size.height as i32)
            .inspect_err(|error| eprintln!("A drop catcher could not be put up: {error}"))
            .ok()
        })
        .collect(),
    )
  }

  pub(in crate::references) fn put_away(self) {
    for window in self.0 {
      let _ = unsafe { RevokeDragDrop(window) };
      let _ = unsafe { DestroyWindow(window) };
    }
  }
}

fn catcher(x: i32, y: i32, width: i32, height: i32) -> Result<HWND, String> {
  let window = unsafe {
    CreateWindowExW(
      WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
      CLASS,
      PCWSTR::null(),
      WS_POPUP,
      x,
      y,
      width,
      height,
      None,
      Some(HMENU::default()),
      None,
      None,
    )
  }
  .map_err(|error| error.to_string())?;
  let ready = unsafe { SetLayeredWindowAttributes(window, COLORREF(0), ALPHA, LWA_ALPHA) }
    .map_err(|error| error.to_string())
    .and_then(|()| overlay_surface::set_capture_affinity(window, false))
    .and_then(|()| {
      let target: IDropTarget = Target {
        picture: Cell::new(false),
      }
      .into();
      unsafe { RegisterDragDrop(window, &target) }.map_err(|error| error.to_string())
    });
  if let Err(error) = ready {
    let _ = unsafe { DestroyWindow(window) };
    return Err(error);
  }
  let _ = unsafe { ShowWindow(window, SW_SHOWNOACTIVATE) };
  Ok(window)
}

unsafe extern "system" fn window_proc(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match message {
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
  }
}

/// Runs `work` on the main thread once OLE's call has returned.
fn later(work: impl FnOnce() + Send + 'static) {
  if let Some(app) = super::APP.get() {
    let _ = app.run_on_main_thread(work);
  }
}

#[implement(IDropTarget)]
struct Target {
  /// Whether the drag over the catcher carries a picture, settled as it
  /// enters.
  picture: Cell<bool>,
}

impl Target {
  fn effect(&self) -> DROPEFFECT {
    if self.picture.get() {
      DROPEFFECT_COPY
    } else {
      DROPEFFECT_NONE
    }
  }
}

/// Writes `effect` through the pointer OLE hands over, which may be null.
fn answer(out: *mut DROPEFFECT, effect: DROPEFFECT) {
  if let Some(out) = unsafe { out.as_mut() } {
    *out = effect;
  }
}

/// The address `data` offers, where it is a picture's on the web.
fn picture_address(data: &IDataObject) -> Option<String> {
  dragged_windows::address(data).filter(|address| is_picture_address(address))
}

/// Whether `data` holds a picture in a form a reference can open.
fn carries_picture(data: &IDataObject) -> bool {
  dragged_windows::holds_picture(data) || picture_address(data).is_some()
}

/// The picture `data` holds in every form a reference can open, the one
/// that keeps the most of it first, its address on the web last.
fn forms(data: &IDataObject) -> Vec<Dropped> {
  let mut forms: Vec<Dropped> = dragged_windows::picture(data)
    .map(|picture| match picture {
      DroppedPicture::File(path) => Dropped::File(path),
      DroppedPicture::Data(bytes) => Dropped::Data(bytes),
    })
    .into_iter()
    .collect();
  forms.extend(picture_address(data).map(Dropped::Address));
  forms
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for Target_Impl {
  fn DragEnter(
    &self,
    data: Ref<'_, IDataObject>,
    _keys: MODIFIERKEYS_FLAGS,
    _point: &POINTL,
    effect: *mut DROPEFFECT,
  ) -> ComResult<()> {
    let picture = data.as_ref().is_some_and(carries_picture);
    self.picture.set(picture);
    answer(effect, self.effect());
    later(move || drop_watch::over_catcher(picture));
    Ok(())
  }

  fn DragOver(
    &self,
    _keys: MODIFIERKEYS_FLAGS,
    _point: &POINTL,
    effect: *mut DROPEFFECT,
  ) -> ComResult<()> {
    answer(effect, self.effect());
    Ok(())
  }

  fn DragLeave(&self) -> ComResult<()> {
    self.picture.set(false);
    Ok(())
  }

  fn Drop(
    &self,
    data: Ref<'_, IDataObject>,
    _keys: MODIFIERKEYS_FLAGS,
    point: &POINTL,
    effect: *mut DROPEFFECT,
  ) -> ComResult<()> {
    answer(effect, DROPEFFECT_NONE);
    if !self.picture.replace(false) {
      return Ok(());
    }
    let Some(data) = data.as_ref() else {
      return Ok(());
    };
    let forms = forms(data);
    if forms.is_empty() {
      return Ok(());
    }
    let scale = scale_at(POINT {
      x: point.x,
      y: point.y,
    });
    let at = Position::Physical(PhysicalPosition::new(point.x, point.y));
    answer(effect, DROPEFFECT_COPY);
    later(move || drop_watch::dropped(forms, at, scale));
    Ok(())
  }
}
