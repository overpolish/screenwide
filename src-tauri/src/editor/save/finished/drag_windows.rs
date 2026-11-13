// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A file dragged out of a window as Explorer drags one: the shell's own data
//! object for it, which carries its path wherever files are taken, drawn
//! under the pointer as the caller asks.
//!
//! OLE's drag runs a loop of its own on the calling thread until the drop,
//! and where that loop runs decides what the editor hears meanwhile. Inside
//! the command it would run within WebView2's handling of the call, which
//! WebView2 does not support. Inside a callback of the event loop it holds
//! the loop's queue until the drop, and Tauri queues a webview's drag events
//! there, so the editor's page would hear where the drag went only once it
//! ended. It therefore starts from a timer of the event loop's thread, which
//! the loop's message pump calls outside any callback of its own: the queue
//! keeps moving through the drag's loop as it would through the loop's own.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use tauri::Manager;

use windows::core::HSTRING;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
  CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::System::Com::{CoCreateInstance, IDataObject, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Ole::DROPEFFECT_COPY;
use windows::Win32::UI::Shell::{
  BHID_DataObject, CLSID_DragDropHelper, IDragSourceHelper, IShellItem,
  SHCreateItemFromParsingName, SHDoDragDrop, SHDRAGIMAGE,
};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use super::DragLook;

/// No colour of the drag image is see-through by key; its alpha says so.
const CLR_NONE: COLORREF = COLORREF(0xFFFF_FFFF);

/// What is drawn under the pointer: premultiplied BGRA rows, the top first.
struct Drawn {
  bgra: Vec<u8>,
  height: u32,
  width: u32,
}

/// A drag waiting for its timer.
struct Pending {
  drawn: Option<Drawn>,
  path: PathBuf,
  window: tauri::WebviewWindow,
}

thread_local! {
  /// The drag the event loop's thread starts when its timer fires.
  static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

pub(super) fn drag(
  window: &tauri::WebviewWindow,
  path: &Path,
  look: &DragLook,
) -> Result<(), String> {
  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  let window = window.clone();
  let path = path.to_owned();
  let look = look.clone();
  // Decoded off the event loop, then handed to its thread to start.
  std::thread::spawn(move || {
    let drawn = drawn(&path, &look, scale);
    let started = window.clone().run_on_main_thread(move || {
      PENDING.with(|pending| {
        pending.replace(Some(Pending {
          drawn,
          path,
          window,
        }))
      });
      if unsafe { SetTimer(None, 0, 0, Some(start_pending)) } == 0 {
        PENDING.with(|pending| pending.take());
        eprintln!("Could not start the drag: no timer");
      }
    });
    if let Err(error) = started {
      eprintln!("Could not start the drag: {error}");
    }
  });
  Ok(())
}

unsafe extern "system" fn start_pending(_window: HWND, _message: u32, timer: usize, _time: u32) {
  let _ = unsafe { KillTimer(None, timer) };
  let Some(pending) = PENDING.with(|pending| pending.take()) else {
    return;
  };
  if let Err(error) = start(&pending.window, &pending.path, pending.drawn.as_ref()) {
    eprintln!("Could not drag {} out: {error}", pending.path.display());
  }
}

/// The picture `look` asks for at `scale` pixels to the point, or none for
/// the file's icon, which the shell draws itself.
fn drawn(path: &Path, look: &DragLook, scale: f64) -> Option<Drawn> {
  let (picture, width, height) = match look {
    DragLook::Icon => return None,
    DragLook::File { width, height } => (image::open(path).ok()?, *width, *height),
    DragLook::Png { png, width, height } => (image::load_from_memory(png).ok()?, *width, *height),
  };
  let pixels = |points: f64| (points * scale).round().max(1.0) as u32;
  let (width, height) = (pixels(width), pixels(height));
  let rgba = image::imageops::resize(
    &picture.to_rgba8(),
    width,
    height,
    image::imageops::FilterType::Triangle,
  );
  let mut bgra = rgba.into_raw();
  for pixel in bgra.chunks_exact_mut(4) {
    let alpha = u16::from(pixel[3]);
    let premultiplied = |channel: u8| ((u16::from(channel) * alpha + 127) / 255) as u8;
    let (red, green, blue) = (pixel[0], pixel[1], pixel[2]);
    pixel[0] = premultiplied(blue);
    pixel[1] = premultiplied(green);
    pixel[2] = premultiplied(red);
  }
  Some(Drawn {
    bgra,
    height,
    width,
  })
}

fn bitmap(drawn: &Drawn) -> Result<HBITMAP, String> {
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: drawn.width as i32,
      // Negative: the rows run from the top.
      biHeight: -(drawn.height as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut bits = std::ptr::null_mut();
  let bitmap = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0) }
    .map_err(|error| error.to_string())?;
  // Safety: the section holds `width * height` 32-bit pixels, which is the
  // length of `bgra`.
  unsafe {
    std::ptr::copy_nonoverlapping(drawn.bgra.as_ptr(), bits.cast::<u8>(), drawn.bgra.len());
  }
  Ok(bitmap)
}

/// Draws `drawn` under the pointer for the drag of `data`, centred on it.
fn set_image(data: &IDataObject, drawn: &Drawn) -> Result<(), String> {
  let helper: IDragSourceHelper =
    unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }
      .map_err(|error| error.to_string())?;
  let bitmap = bitmap(drawn)?;
  let image = SHDRAGIMAGE {
    sizeDragImage: SIZE {
      cx: drawn.width as i32,
      cy: drawn.height as i32,
    },
    ptOffset: POINT {
      x: drawn.width as i32 / 2,
      y: drawn.height as i32 / 2,
    },
    hbmpDragImage: bitmap,
    crColorKey: CLR_NONE,
  };
  // The helper owns the bitmap once it takes it, and only then.
  unsafe { helper.InitializeFromBitmap(&image, data) }.map_err(|error| {
    let _ = unsafe { DeleteObject(bitmap.into()) };
    error.to_string()
  })
}

fn start(window: &tauri::WebviewWindow, path: &Path, drawn: Option<&Drawn>) -> Result<(), String> {
  let hwnd = HWND(window.hwnd().map_err(|error| error.to_string())?.0);
  let item: IShellItem = unsafe { SHCreateItemFromParsingName(&HSTRING::from(path), None) }
    .map_err(|error| error.to_string())?;
  let data: IDataObject =
    unsafe { item.BindToHandler(None, &BHID_DataObject) }.map_err(|error| error.to_string())?;
  if let Some(drawn) = drawn {
    // Without its picture the drag still carries the file, drawn as its icon.
    if let Err(error) = set_image(&data, drawn) {
      eprintln!("The drag is drawn as the file's icon: {error}");
    }
  }
  // A drag out of a window that lets clicks through around its parts, as
  // the editor's island window does, lets the pointer through all of it.
  let app = window.app_handle();
  let hit_area = crate::app_windows::options::hit_area::set_dragging_out;
  hit_area(app, window.label(), true);
  let result = unsafe { SHDoDragDrop(Some(hwnd), &data, None, DROPEFFECT_COPY) };
  hit_area(app, window.label(), false);
  result.map(|_| ()).map_err(|error| error.to_string())
}
