// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures dropped on the workspace: the editor window's OLE drop target.
//!
//! OLE calls the target on the thread that registered it, the event loop's,
//! which tao has initialised for OLE. Everything a drop needs is read during
//! the call, since the dragged data is only there for its length.

use crate::editor::images::dragged_windows as formats;

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
use crate::editor::images::dropped::{Carried, DroppedPicture, ImagePoint, SoundDrag};

#[implement(IDropTarget)]
struct ImageDropTarget {
  hwnd: HWND,
  /// Whether the drag over the window now holds a picture the workspace can
  /// take, settled as it enters.
  accepts: Cell<bool>,
  /// Whether the drag over the window holds a sound, which it announced.
  sound: Cell<bool>,
}

/// Makes the editor window `hwnd` a drop target. A window that cannot be one
/// still works for everything else.
pub(super) fn register(hwnd: HWND) {
  let target: IDropTarget = ImageDropTarget {
    hwnd,
    accepts: Cell::new(false),
    sound: Cell::new(false),
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

  /// Hands what was carried to the editor's drop callback.
  fn hand(&self, carried: Carried) -> bool {
    let Some(engine) = self.surface() else {
      return false;
    };
    let Ok(mut callback) = windows(&engine.inner).image_drop.lock() else {
      return false;
    };
    let Some(callback) = callback.as_mut() else {
      return false;
    };
    callback(carried);
    true
  }

  /// Tells the editor a sound announced over the window has gone.
  fn sound_left(&self) {
    if self.sound.replace(false) {
      self.hand(Carried::Sound(SoundDrag::Left));
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
    // The picture takes no sound: songs and sound effects are dropped on the
    // timeline, in the page, so a sound arriving here is only announced, for
    // the editor to show where it goes. OLE tells a target that refused of
    // the drag being let go over it as if it had left.
    let carried = data
      .as_ref()
      .and_then(formats::picture)
      .filter(|_| self.surface().is_some());
    match carried {
      Some(DroppedPicture::File(path)) if screenwide_sound_track::store::is_sound_file(&path) => {
        self.accepts.set(false);
        self.sound.set(true);
        self.hand(Carried::Sound(SoundDrag::Over(path)));
      }
      carried => self.accepts.set(carried.is_some()),
    }
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
    self.sound_left();
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
    self.sound_left();
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
    if self.hand(Carried::Picture(picture, at)) {
      answer(effect, DROPEFFECT_COPY);
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
