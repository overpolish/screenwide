// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures dropped on the workspace: the editor window's OLE drop target.
//! The twin of `recording_preview_surface_macos+drop.m`.
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

use super::annotation::{layer_image_rect, take_drawing_layer};
use super::*;
use crate::editor::annotations::gesture::drawing_layer;
use crate::editor::preview_platform::StickerDropCallback;
use crate::editor::stickers::dropped::StickerPoint;

#[implement(IDropTarget)]
struct StickerDropTarget {
  hwnd: HWND,
  /// Whether the drag over the window now holds a picture the workspace can
  /// take, settled as it enters.
  accepts: Cell<bool>,
}

/// Makes the editor window `hwnd` a drop target. A window that cannot be one
/// still works for everything else.
pub(super) fn register(hwnd: HWND) {
  let target: IDropTarget = StickerDropTarget {
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

/// Where a drop at `client`, in the editor window's pixels, lands: the
/// topmost picture under it and the point in its image-normalised space,
/// with the layer taken in hand where that changed the choice, as a press
/// with the sticker tool there would.
fn landing(inner: &SurfaceInner, client: (f64, f64)) -> (Option<StickerPoint>, Option<u32>) {
  let Ok(mut state) = inner.state.lock() else {
    return (None, None);
  };
  let scale = state.scale.max(0.1);
  let point = (client.0 / scale, client.1 / scale);
  let pictures: Vec<_> = state
    .selection_targets
    .iter()
    .filter_map(|target| {
      let rect = layer_image_rect(&state, target.layer_id as i32)?;
      Some((target.layer_id, [rect.x, rect.y, rect.width, rect.height]))
    })
    .collect();
  let at = drawing_layer(pictures.into_iter(), point).and_then(|layer| {
    let image = layer_image_rect(&state, layer as i32)?;
    (image.width > 0.0 && image.height > 0.0).then(|| StickerPoint {
      layer,
      x: (point.0 - image.x) / image.width,
      y: (point.1 - image.y) / image.height,
    })
  });
  let taken = take_drawing_layer(inner, &mut state, point);
  (at, taken)
}

impl StickerDropTarget {
  /// The surface, while it takes drops: an editor showing, not covered by
  /// the frontend's own chrome, with somewhere to hand the picture.
  fn surface(&self) -> Option<std::sync::Arc<SurfaceInner>> {
    let inner = surface_for_editor(self.hwnd)?;
    let taking = inner.callbacks.lock().ok()?.sticker_drop.is_some()
      && !inner.editor.is_suspended()
      && inner.state.lock().ok()?.editor_active;
    taking.then_some(inner)
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
impl IDropTarget_Impl for StickerDropTarget_Impl {
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
    let Some(inner) = self.surface() else {
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
    let (at, taken) = landing(&inner, (f64::from(client.x), f64::from(client.y)));
    if let Some(layer) = taken {
      emit_selection(&inner, Some(layer));
    }
    if let Ok(mut callbacks) = inner.callbacks.lock() {
      if let Some(callback) = callbacks.sticker_drop.as_mut() {
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
  pub(crate) fn set_sticker_drop_callback(&mut self, callback: StickerDropCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.sticker_drop = Some(callback);
    }
  }
}
