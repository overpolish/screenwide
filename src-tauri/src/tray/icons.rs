// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::image::Image;
#[cfg(target_os = "windows")]
use tauri::menu::ContextMenu;

#[cfg(target_os = "windows")]
mod assets {
  pub const ANNOTATE: &[u8] = include_bytes!("../../icons/menu/windows/annotate.png");
  pub const CANCEL: &[u8] = include_bytes!("../../icons/menu/windows/cancel.png");
  pub const CLIPBOARD: &[u8] = include_bytes!("../../icons/menu/windows/clipboard.png");
  pub const DISCARD: &[u8] = include_bytes!("../../icons/menu/windows/discard.png");
  pub const OPEN: &[u8] = include_bytes!("../../icons/menu/windows/open.png");
  pub const PAUSE: &[u8] = include_bytes!("../../icons/menu/windows/pause.png");
  pub const QUIT: &[u8] = include_bytes!("../../icons/menu/windows/quit.png");
  pub const PROJECT: &[u8] = include_bytes!("../../icons/menu/windows/project.png");
  pub const REPLAY: &[u8] = include_bytes!("../../icons/menu/windows/replay.png");
  pub const RESUME: &[u8] = include_bytes!("../../icons/menu/windows/resume.png");
  pub const RULER: &[u8] = include_bytes!("../../icons/menu/windows/ruler.png");
  pub const SETTINGS: &[u8] = include_bytes!("../../icons/menu/windows/settings.png");
  pub const STOP: &[u8] = include_bytes!("../../icons/menu/windows/stop.png");
  pub const TEXT: &[u8] = include_bytes!("../../icons/menu/windows/text.png");
  pub const TIMER: &[u8] = include_bytes!("../../icons/menu/windows/timer.png");
}

#[cfg(not(target_os = "windows"))]
mod assets {
  pub const ANNOTATE: &[u8] = include_bytes!("../../icons/menu/annotate.png");
  pub const CANCEL: &[u8] = include_bytes!("../../icons/menu/cancel.png");
  pub const CLIPBOARD: &[u8] = include_bytes!("../../icons/menu/clipboard.png");
  pub const DISCARD: &[u8] = include_bytes!("../../icons/menu/discard.png");
  pub const OPEN: &[u8] = include_bytes!("../../icons/menu/open.png");
  pub const PAUSE: &[u8] = include_bytes!("../../icons/menu/pause.png");
  pub const QUIT: &[u8] = include_bytes!("../../icons/menu/quit.png");
  pub const PROJECT: &[u8] = include_bytes!("../../icons/menu/project.png");
  pub const REPLAY: &[u8] = include_bytes!("../../icons/menu/replay.png");
  pub const RESUME: &[u8] = include_bytes!("../../icons/menu/resume.png");
  pub const RULER: &[u8] = include_bytes!("../../icons/menu/ruler.png");
  pub const SETTINGS: &[u8] = include_bytes!("../../icons/menu/settings.png");
  pub const STOP: &[u8] = include_bytes!("../../icons/menu/stop.png");
  pub const TEXT: &[u8] = include_bytes!("../../icons/menu/text.png");
  pub const TIMER: &[u8] = include_bytes!("../../icons/menu/timer.png");
}
pub use assets::*;

pub fn load(bytes: &'static [u8]) -> tauri::Result<Image<'static>> {
  let image = Image::from_bytes(bytes)?;
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::Graphics::Gdi::GetSysColor;
    let color = if high_contrast() {
      (unsafe {
        windows::Win32::Graphics::Gdi::GetSysColor(windows::Win32::Graphics::Gdi::COLOR_MENUTEXT)
      }) as u32
    } else {
      match menu_is_dark() {
        Some(true) => 0xFFFFFF,
        Some(false) => 0,
        None => (unsafe { GetSysColor(windows::Win32::Graphics::Gdi::COLOR_MENUTEXT) }) as u32,
      }
    };
    let mut rgba = image.rgba().to_vec();
    for pixel in rgba.as_chunks_mut::<4>().0 {
      pixel[0] = color as u8;
      pixel[1] = (color >> 8) as u8;
      pixel[2] = (color >> 16) as u8;
    }
    Ok(Image::new_owned(rgba, image.width(), image.height()))
  }
  #[cfg(not(target_os = "windows"))]
  Ok(image)
}

#[cfg(target_os = "windows")]
pub fn apply_system_foreground(image: Image<'static>) -> Image<'static> {
  let color = if high_contrast() {
    let value = unsafe {
      windows::Win32::Graphics::Gdi::GetSysColor(windows::Win32::Graphics::Gdi::COLOR_WINDOWTEXT)
    } as u32;
    [
      (value & 0xff) as u8,
      ((value >> 8) & 0xff) as u8,
      ((value >> 16) & 0xff) as u8,
    ]
  } else if tray_is_dark().unwrap_or(false) {
    [255, 255, 255]
  } else {
    [0, 0, 0]
  };
  let mut rgba = image.rgba().to_vec();
  for pixel in rgba.as_chunks_mut::<4>().0 {
    if pixel[3] != 0 {
      pixel[..3].copy_from_slice(&color);
    }
  }
  Image::new_owned(rgba, image.width(), image.height())
}

#[cfg(target_os = "windows")]
fn theme_value(name: windows::core::PCWSTR) -> Option<bool> {
  use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
  let key = windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
  let mut value = 0u32;
  let mut size = std::mem::size_of::<u32>() as u32;
  let result = unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      key,
      name,
      RRF_RT_REG_DWORD,
      None,
      Some((&mut value as *mut u32).cast()),
      Some(&mut size),
    )
  };
  result.is_ok().then_some(value == 0)
}

#[cfg(target_os = "windows")]
fn menu_is_dark() -> Option<bool> {
  theme_value(windows::core::w!("AppsUseLightTheme"))
}

#[cfg(target_os = "windows")]
fn tray_is_dark() -> Option<bool> {
  theme_value(windows::core::w!("SystemUsesLightTheme"))
}

#[cfg(target_os = "windows")]
fn high_contrast() -> bool {
  use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
  use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETHIGHCONTRAST};
  let mut settings = HIGHCONTRASTW {
    cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
    ..Default::default()
  };
  unsafe {
    SystemParametersInfoW(
      SPI_GETHIGHCONTRAST,
      settings.cbSize,
      Some((&mut settings as *mut HIGHCONTRASTW).cast()),
      Default::default(),
    )
  }
  .is_ok()
    && settings.dwFlags.contains(HCF_HIGHCONTRASTON)
}

#[cfg(target_os = "windows")]
pub fn apply_menu_style(menu: &tauri::menu::Menu<tauri::Wry>) {
  use windows::Win32::UI::WindowsAndMessaging::{
    GetMenuInfo, SetMenuInfo, MENUINFO, MIM_STYLE, MNS_CHECKORBMP,
  };

  let Ok(handle) = menu.hpopupmenu() else {
    return;
  };
  let mut info = MENUINFO {
    cbSize: std::mem::size_of::<MENUINFO>() as u32,
    fMask: MIM_STYLE,
    ..Default::default()
  };
  if unsafe {
    GetMenuInfo(
      windows::Win32::UI::WindowsAndMessaging::HMENU(handle as *mut _),
      &mut info,
    )
  }
  .is_err()
  {
    return;
  }
  info.dwStyle |= MNS_CHECKORBMP;
  let _ = unsafe {
    SetMenuInfo(
      windows::Win32::UI::WindowsAndMessaging::HMENU(handle as *mut _),
      &info,
    )
  };
}

/// Template images let AppKit supply light/dark, selected, and disabled menu
/// colours, and macOS 27 hides menu item images unless the item asks for
/// them; muda sets neither. The tray hands its menu to the status item only
/// while the menu is open, so the items are prepared as tracking begins.
#[cfg(target_os = "macos")]
pub fn show_menu_images(tray: &tauri::tray::TrayIcon) -> tauri::Result<()> {
  use std::ptr::NonNull;

  use block2::RcBlock;
  use objc2::runtime::NSObjectProtocol;
  use objc2::{msg_send, sel};
  use objc2_app_kit::NSMenuDidBeginTrackingNotification;
  use objc2_foundation::{NSNotification, NSNotificationCenter};

  tray.with_inner_tray_icon(|inner| {
    let Some(status_item) = inner.ns_status_item() else {
      return;
    };
    let block = RcBlock::new(move |notification: NonNull<NSNotification>| {
      let Some(mtm) = objc2::MainThreadMarker::new() else {
        return;
      };
      let Some(menu) = status_item.menu(mtm) else {
        return;
      };
      // Safety: the notification center passes a live notification.
      let notification = unsafe { notification.as_ref() };
      if !notification
        .object()
        .is_some_and(|object| std::ptr::addr_eq(&*object, &*menu))
      {
        return;
      }
      let visibility = sel!(setPreferredImageVisibility:);
      for item in menu.itemArray() {
        let Some(image) = item.image() else {
          continue;
        };
        image.setTemplate(true);
        if item.respondsToSelector(visibility) {
          // Safety: NSMenuItem's own property on macOS 27 and later, checked
          // above; 1 is NSMenuItemImageVisibilityVisible.
          let _: () = unsafe { msg_send![&*item, setPreferredImageVisibility: 1isize] };
        }
      }
    });
    // Posted on the main thread, where AppKit tracks menus. The notification
    // center retains the observer for the process lifetime; the tray installs
    // it once.
    unsafe {
      NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
        Some(NSMenuDidBeginTrackingNotification),
        None,
        None,
        &block,
      );
    }
  })
}
