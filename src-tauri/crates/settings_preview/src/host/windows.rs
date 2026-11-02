// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a preview is drawn on Windows: a DirectComposition visual on the
//! window that holds the webview, the twin of the macOS layer. A tool that
//! draws its screen goes in the window's tree under WebView2's child window,
//! as the editor's preview does, and shows through a transparent box; one
//! that draws over the page goes in the topmost tree, above that child.
//! Visuals take no input, so the page keeps every event.
//!
//! A window has one target of each kind, so every preview in it shares the
//! window's composition, each in a clip, frame and picture visual of its own.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, Weak};

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::{
  DCompositionCreateDevice, IDCompositionDevice, IDCompositionRectangleClip, IDCompositionTarget,
  IDCompositionVisual, DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

use super::Stacking;
use crate::placement::{PreviewArea, PreviewPlacement};

/// The window a preview is drawn in.
pub struct HostWindow {
  /// The window that holds the webview, as Tauri hands it over.
  pub hwnd: isize,
}

/// One window's composition and its two trees.
struct Composition {
  device: IDCompositionDevice,
  below: (IDCompositionTarget, IDCompositionVisual),
  above: (IDCompositionTarget, IDCompositionVisual),
}

// The interfaces are process-wide tokens, used behind the registry's lock or
// the slot's own.
unsafe impl Send for Composition {}
unsafe impl Sync for Composition {}

impl Composition {
  fn new(hwnd: HWND) -> windows::core::Result<Self> {
    // Every visual's content is a swap chain, so the device needs no
    // Direct3D device of its own.
    let device: IDCompositionDevice = unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }?;
    let tree = |topmost: bool| -> windows::core::Result<_> {
      let target = unsafe { device.CreateTargetForHwnd(hwnd, topmost) }?;
      let root = unsafe { device.CreateVisual() }?;
      unsafe { target.SetRoot(&root) }?;
      Ok((target, root))
    };
    Ok(Self {
      below: tree(false)?,
      above: tree(true)?,
      device,
    })
  }
}

/// Every window's composition, by its handle, while a preview uses it.
static COMPOSITIONS: LazyLock<Mutex<HashMap<isize, Weak<Composition>>>> =
  LazyLock::new(Default::default);

fn composition(hwnd: isize) -> Result<Arc<Composition>, String> {
  let mut compositions = COMPOSITIONS
    .lock()
    .map_err(|_| "The preview compositions are unavailable".to_owned())?;
  if let Some(composition) = compositions.get(&hwnd).and_then(Weak::upgrade) {
    return Ok(composition);
  }
  let composition = Arc::new(
    Composition::new(HWND(hwnd as *mut std::ffi::c_void))
      .map_err(|error| format!("DirectComposition could not open for a preview: {error}"))?,
  );
  compositions.retain(|_, composition| composition.strong_count() > 0);
  compositions.insert(hwnd, Arc::downgrade(&composition));
  Ok(composition)
}

/// One preview's visuals.
pub(super) struct Slot {
  composition: Arc<Composition>,
  /// The tree the preview is in.
  root: IDCompositionVisual,
  clip: IDCompositionVisual,
  clip_shape: IDCompositionRectangleClip,
  frame: IDCompositionVisual,
  frame_shape: IDCompositionRectangleClip,
  picture: IDCompositionVisual,
}

unsafe impl Send for Slot {}
unsafe impl Sync for Slot {}

impl Drop for Slot {
  fn drop(&mut self) {
    unsafe {
      let _ = self.root.RemoveVisual(&self.clip);
      let _ = self.composition.device.Commit();
    }
  }
}

pub(super) fn create(
  window: &HostWindow,
  stacking: Stacking,
) -> Result<(Slot, screenwide_gpu::surface::Surface), String> {
  let gpu = screenwide_gpu::shared()?;
  let composition = composition(window.hwnd)?;
  let device = &composition.device;
  let root = match stacking {
    Stacking::Below => composition.below.1.clone(),
    Stacking::Above => composition.above.1.clone(),
  };
  let made = (|| -> windows::core::Result<_> {
    let clip = unsafe { device.CreateVisual() }?;
    let clip_shape = unsafe { device.CreateRectangleClip() }?;
    let frame = unsafe { device.CreateVisual() }?;
    let frame_shape = unsafe { device.CreateRectangleClip() }?;
    let picture = unsafe { device.CreateVisual() }?;
    unsafe {
      clip.SetClip(&clip_shape)?;
      frame.SetClip(&frame_shape)?;
      // DirectComposition samples nearest neighbour unless told otherwise.
      picture.SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)?;
      frame.AddVisual(&picture, false, None::<&IDCompositionVisual>)?;
      clip.AddVisual(&frame, false, None::<&IDCompositionVisual>)?;
      root.AddVisual(&clip, true, None::<&IDCompositionVisual>)?;
    }
    Ok((clip, clip_shape, frame, frame_shape, picture))
  })()
  .map_err(|error| format!("A preview's visuals could not be made: {error}"))?;
  let (clip, clip_shape, frame, frame_shape, picture) = made;
  let surface = screenwide_gpu::surface::Surface::on_visual(gpu, &picture)?;
  let slot = Slot {
    composition,
    root,
    clip,
    clip_shape,
    frame,
    frame_shape,
    picture,
  };
  slot.place(PreviewPlacement::hidden());
  Ok((slot, surface))
}

impl Slot {
  /// Puts the drawing at `placement`, in the page's points, which start at
  /// the client area's corner where WebView2 fills it.
  pub(super) fn place(&self, placement: PreviewPlacement) {
    let scale = placement.scale;
    let pixels = |points: f64| (points * scale) as f32;
    let shown = if placement.visible {
      placement.seen_halfway_faded()
    } else {
      PreviewArea::default()
    };
    let PreviewPlacement {
      frame,
      picture,
      radius,
      ..
    } = placement;
    let radius = pixels(radius);
    let _ = (|| -> windows::core::Result<()> {
      unsafe {
        let clip = &self.clip_shape;
        clip.SetLeft2(pixels(shown.x))?;
        clip.SetTop2(pixels(shown.y))?;
        clip.SetRight2(pixels(shown.right()))?;
        clip.SetBottom2(pixels(shown.bottom()))?;
        self.frame.SetOffsetX2(pixels(frame.x))?;
        self.frame.SetOffsetY2(pixels(frame.y))?;
        let shape = &self.frame_shape;
        shape.SetLeft2(0.0)?;
        shape.SetTop2(0.0)?;
        shape.SetRight2(pixels(frame.width))?;
        shape.SetBottom2(pixels(frame.height))?;
        shape.SetTopLeftRadiusX2(radius)?;
        shape.SetTopLeftRadiusY2(radius)?;
        shape.SetTopRightRadiusX2(radius)?;
        shape.SetTopRightRadiusY2(radius)?;
        shape.SetBottomRightRadiusX2(radius)?;
        shape.SetBottomRightRadiusY2(radius)?;
        shape.SetBottomLeftRadiusX2(radius)?;
        shape.SetBottomLeftRadiusY2(radius)?;
        self.picture.SetOffsetX2(pixels(picture.x - frame.x))?;
        self.picture.SetOffsetY2(pixels(picture.y - frame.y))?;
        self.composition.device.Commit()
      }
    })();
  }
}
