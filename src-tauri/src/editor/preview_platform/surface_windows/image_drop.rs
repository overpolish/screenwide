// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures dropped on the workspace: the editor window's OLE drop target.
//!
//! OLE calls the target on the thread that registered it, the event loop's,
//! which tao has initialised for OLE. Everything a drop needs is read during
//! the call, since the dragged data is only there for its length.

mod formats;

use std::cell::Cell;

use windows::core::{implement, Ref, Result as ComResult};
use windows::Win32::Foundation::{POINT, POINTL};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::Com::IDataObject;
use windows::Win32::System::Ole::{
  IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, DROPEFFECT, DROPEFFECT_COPY,
  DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;

use super::*;
use crate::editor::images::dropped::ImagePoint;

#[implement(IDropTarget)]
struct ImageDropTarget {
  hwnd: HWND,
  /// Whether the drag over the window now holds a picture the workspace can
  /// take, settled as it enters.
  accepts: Cell<bool>,
}

/// Makes the editor window `hwnd` a drop target. A window that cannot be one
/// still works for everything else.
pub(super) fn register(hwnd: HWND) {
  let target: IDropTarget = ImageDropTarget {
    hwnd,
    accepts: Cell::new(false),
  }
  .into();
  if let Err(error) = unsafe { RegisterDragDrop(hwnd, &target) } {
    eprintln!("The preview cannot take dropped pictures: {error}");
  }
}

/// Lets the editor window `hwnd` go as a drop target, before it is destroyed.
pub(super) fn revoke(hwnd: HWND) {
  let _ = unsafe { RevokeDragDrop(hwnd) };
}

impl ImageDropTarget {
  /// The surface, while it takes drops: an editor showing, not covered by
  /// the frontend's own chrome, with somewhere to hand the picture.
  fn surface(&self) -> Option<WorkspaceEditor> {
    let engine = engine_for(self.hwnd)?;
    let taking = windows(&engine.inner)
      .image_drop
      .lock()
      .is_ok_and(|callback| callback.is_some())
      && engine.takes_drops();
    taking.then_some(engine)
  }

  fn effect(&self) -> DROPEFFECT {
    if self.accepts.get() {
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

#[allow(non_snake_case)]
impl IDropTarget_Impl for ImageDropTarget_Impl {
  fn DragEnter(
    &self,
    data: Ref<'_, IDataObject>,
    _keys: MODIFIERKEYS_FLAGS,
    _point: &POINTL,
    effect: *mut DROPEFFECT,
  ) -> ComResult<()> {
    let accepts = self.surface().is_some() && data.as_ref().is_some_and(formats::holds_picture);
    self.accepts.set(accepts);
    answer(effect, self.effect());
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
    self.accepts.set(false);
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
    if !self.accepts.replace(false) {
      return Ok(());
    }
    let Some(engine) = self.surface() else {
      return Ok(());
    };
    let Some(picture) = data.as_ref().and_then(formats::picture) else {
      return Ok(());
    };
    let mut client = POINT {
      x: point.x,
      y: point.y,
    };
    let _ = unsafe { ScreenToClient(self.hwnd, &mut client) };
    let at = engine
      .land_drop((f64::from(client.x), f64::from(client.y)))
      .map(|(layer, x, y)| ImagePoint { layer, x, y });
    if let Ok(mut callback) = windows(&engine.inner).image_drop.lock() {
      if let Some(callback) = callback.as_mut() {
        callback(picture, at);
        answer(effect, DROPEFFECT_COPY);
      }
    }
    Ok(())
  }
}

impl RecordingPreviewSurface {
  /// Installs the callback a picture dropped on the workspace is handed to.
  /// Without one, nothing can be dropped there.
  pub(crate) fn set_image_drop_callback(&mut self, callback: ImageDropCallback) {
    if let Ok(mut slot) = windows(&self.inner).image_drop.lock() {
      *slot = Some(callback);
    }
  }
}
