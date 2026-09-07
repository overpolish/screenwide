// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use tauri::menu::ContextMenu;
use tauri::menu::MenuItemKind;
use tauri::Wry;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
  CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONEAREST};

#[cfg(test)]
#[path = "windows_menu_tests.rs"]
mod tests;
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
  GetCursorPos, SetMenuItemInfoW, HMENU, MENUITEMINFOW, MIIM_BITMAP,
};

struct RegisteredMenu {
  menu: tauri::menu::Menu<Wry>,
  handle: isize,
  bitmaps: Vec<(u32, HBITMAP)>,
}
thread_local! { static MENU: RefCell<Option<RegisteredMenu>> = const { RefCell::new(None) }; }

pub fn register(menu: &tauri::menu::Menu<Wry>) {
  if let Ok(handle) = menu.hpopupmenu() {
    MENU.with(|menus| {
      let previous = menus.borrow_mut().replace(RegisteredMenu {
        menu: menu.clone(),
        handle,
        bitmaps: Vec::new(),
      });
      if let Some(previous) = previous {
        release(previous);
      }
    });
  }
}

pub fn prepare(menu: HMENU) {
  let handle = menu.0 as isize;
  if !MENU.with(|registered| {
    registered
      .borrow()
      .as_ref()
      .is_some_and(|menu| menu.handle == handle)
  }) {
    return;
  }
  let dpi = dpi_for_cursor().unwrap_or(96);
  let size = (16u32 * dpi + 48) / 96;
  let retained = MENU.with(|registered| {
    registered
      .borrow()
      .as_ref()
      .map(|registered| registered.menu.clone())
  });
  let items = retained
    .and_then(|menu| menu.items().ok())
    .unwrap_or_default();
  let mut installed = Vec::new();
  for (position, item) in items.iter().enumerate() {
    let position = position as u32;
    let Some(source) = icon_for(item) else {
      continue;
    };
    let Some(bitmap) = make_bitmap(source, size) else {
      continue;
    };
    let bitmap_info = MENUITEMINFOW {
      cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
      fMask: MIIM_BITMAP,
      hbmpItem: bitmap,
      ..Default::default()
    };
    if let Err(error) = unsafe { SetMenuItemInfoW(menu, position, true, &bitmap_info) } {
      eprintln!("Windows tray: SetMenuItemInfoW failed at position {position}: {error}");
      unsafe {
        let _ = DeleteObject(bitmap.into());
      }
    } else {
      installed.push((position, bitmap));
    }
  }
  MENU.with(|registered| {
    if let Some(menu) = registered.borrow_mut().as_mut() {
      for (position, bitmap) in installed {
        if let Some((_, old)) = menu
          .bitmaps
          .iter_mut()
          .find(|(old_position, _)| *old_position == position)
        {
          let old = std::mem::replace(old, bitmap);
          unsafe {
            let _ = DeleteObject(old.into());
          }
        } else {
          menu.bitmaps.push((position, bitmap));
        }
      }
    }
  });
}

pub fn cleanup() {
  MENU.with(|menu| {
    if let Some(menu) = menu.borrow_mut().take() {
      release(menu);
    }
  });
}

fn release(menu: RegisteredMenu) {
  for (position, bitmap) in menu.bitmaps {
    let info = MENUITEMINFOW {
      cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
      fMask: MIIM_BITMAP,
      hbmpItem: HBITMAP::default(),
      ..Default::default()
    };
    let _ = unsafe { SetMenuItemInfoW(HMENU(menu.handle as *mut _), position, true, &info) };
    unsafe {
      let _ = DeleteObject(bitmap.into());
    }
  }
}

fn dpi_for_cursor() -> Option<u32> {
  let mut point = POINT::default();
  if unsafe { GetCursorPos(&mut point) }.is_err() {
    eprintln!("Windows tray: GetCursorPos failed");
    return None;
  }
  let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
  if monitor.is_invalid() {
    eprintln!("Windows tray: MonitorFromPoint failed");
    return None;
  }
  let mut x = 96;
  let mut y = 96;
  if let Err(error) = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y) } {
    eprintln!("Windows tray: GetDpiForMonitor failed: {error}");
    None
  } else {
    Some(x.max(y))
  }
}

fn icon_for(item: &MenuItemKind<Wry>) -> Option<&'static [u8]> {
  let id = item.id().as_ref();
  Some(if id == super::OPEN_MENU_ID {
    super::icons::OPEN
  } else if id == super::OPEN_CLIPBOARD_SCREENSHOT_MENU_ID {
    super::icons::CLIPBOARD
  } else if id == super::RECOGNIZE_TEXT_MENU_ID {
    super::icons::TEXT
  } else if id == super::RULER_OVERLAY_MENU_ID {
    super::icons::RULER
  } else if id == super::SETTINGS_MENU_ID {
    super::icons::SETTINGS
  } else if id == super::QUIT_MENU_ID {
    super::icons::QUIT
  } else if id == super::PAUSE_MENU_ID {
    if item
      .as_icon_menuitem()
      .and_then(|item| item.text().ok())
      .as_deref()
      == Some("Resume Recording")
    {
      super::icons::RESUME
    } else {
      super::icons::PAUSE
    }
  } else if id == super::STOP_MENU_ID {
    super::icons::STOP
  } else if id == super::DISCARD_MENU_ID {
    if item
      .as_icon_menuitem()
      .and_then(|item| item.text().ok())
      .as_deref()
      == Some("Cancel Recording")
    {
      super::icons::CANCEL
    } else {
      super::icons::DISCARD
    }
  } else {
    return None;
  })
}

fn make_bitmap(source: &'static [u8], size: u32) -> Option<HBITMAP> {
  let source = super::icons::load(source).ok()?;
  let source = image::RgbaImage::from_raw(source.width(), source.height(), source.rgba().to_vec())?;
  let source = image::imageops::resize(&source, size, size, image::imageops::FilterType::Lanczos3);
  let bitmap_info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: size as i32,
      biHeight: -(size as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut bits = std::ptr::null_mut();
  let bitmap =
    match unsafe { CreateDIBSection(None, &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0) } {
      Ok(bitmap) => bitmap,
      Err(error) => {
        eprintln!("Windows tray: CreateDIBSection failed: {error}");
        return None;
      }
    };
  if bits.is_null() {
    eprintln!("Windows tray: CreateDIBSection returned null pixel storage");
    unsafe {
      let _ = DeleteObject(bitmap.into());
    }
    return None;
  }
  let dst =
    unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), (size * size * 4) as usize) };
  for (dst, src) in dst.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
    let a = u32::from(src[3]);
    dst.copy_from_slice(&[
      (u32::from(src[2]) * a / 255) as u8,
      (u32::from(src[1]) * a / 255) as u8,
      (u32::from(src[0]) * a / 255) as u8,
      src[3],
    ]);
  }
  Some(bitmap)
}
