// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "windows")]
mod windows_platform {
  use std::{
    collections::HashSet,
    ffi::{c_void, OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
  };

  use image::{ImageBuffer, Rgba};
  use rapidfuzz::fuzz::ratio;
  use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
      Foundation::{CloseHandle, HWND},
      Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
      },
      System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
      },
      UI::{
        Shell::ExtractIconExW,
        WindowsAndMessaging::{
          DestroyIcon, GetIconInfo, SetWindowPos, ICONINFO, SWP_NOACTIVATE, SWP_NOMOVE,
          SWP_NOZORDER,
        },
      },
    },
  };

  pub fn selectable_window_ids() -> Option<HashSet<u32>> {
    None
  }

  fn find_window(id: u32, pid: u32, title: &str) -> Result<HWND, String> {
    let windows = xcap::Window::all().map_err(|error| error.to_string())?;
    windows
      .into_iter()
      .filter(|window| window.pid().ok() == Some(pid))
      .max_by(|left, right| {
        let score = |window: &xcap::Window| {
          if window.id().ok() == Some(id) {
            f64::MAX
          } else {
            ratio(window.title().unwrap_or_default().chars(), title.chars())
          }
        };
        score(left).total_cmp(&score(right))
      })
      .and_then(|window| window.id().ok())
      .map(|window_id| HWND(window_id as usize as *mut c_void))
      .ok_or_else(|| format!("Could not find window '{title}'"))
  }

  /// Full path of a process's executable image.
  ///
  /// Uses `PROCESS_QUERY_LIMITED_INFORMATION` + `QueryFullProcessImageNameW`
  /// rather than `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ` +
  /// `GetModuleFileNameEx`: the heavier rights are denied when the target runs
  /// at a higher integrity level (e.g. an elevated app while we are not), which
  /// left elevated windows with no icon at all. The limited right crosses that
  /// boundary, and reading the icon afterwards touches the file on disk, not the
  /// process, so it needs nothing more.
  fn process_image_path(pid: u32) -> Option<PathBuf> {
    unsafe {
      let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
      let mut buffer = [0_u16; 1024];
      let mut length = buffer.len() as u32;
      let result = QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_WIN32,
        PWSTR::from_raw(buffer.as_mut_ptr()),
        &mut length,
      );
      let _ = CloseHandle(process);
      result.ok()?;
      (length > 0).then(|| PathBuf::from(OsString::from_wide(&buffer[..length as usize])))
    }
  }

  pub fn app_icon(cache_dir: &Path, pid: u32) -> Option<PathBuf> {
    let executable = process_image_path(pid)?;
    let name = executable.file_stem()?.to_string_lossy();
    let path = cache_dir.join(format!("app-{name}.png"));
    if path.exists() {
      return Some(path);
    }
    unsafe {
      let executable_wide = OsStr::new(&executable)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
      let mut icon = Default::default();
      if ExtractIconExW(
        PCWSTR::from_raw(executable_wide.as_ptr()),
        0,
        Some(&mut icon),
        None,
        1,
      ) == 0
      {
        return None;
      }

      let mut info = ICONINFO::default();
      if GetIconInfo(icon, &mut info).is_err() {
        let _ = DestroyIcon(icon);
        return None;
      }
      let dc = CreateCompatibleDC(None);
      let mut bitmap = BITMAP::default();
      if GetObjectW(
        info.hbmColor.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bitmap as *mut _ as *mut _),
      ) == 0
      {
        cleanup_icon(icon, info, dc);
        return None;
      }
      let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
          biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
          biWidth: bitmap.bmWidth,
          biHeight: -bitmap.bmHeight,
          biPlanes: 1,
          biBitCount: 32,
          biCompression: BI_RGB.0,
          ..Default::default()
        },
        ..Default::default()
      };
      let mut pixels = vec![0_u8; (bitmap.bmWidth * bitmap.bmHeight * 4) as usize];
      let lines = GetDIBits(
        dc,
        info.hbmColor,
        0,
        bitmap.bmHeight as u32,
        Some(pixels.as_mut_ptr().cast()),
        &mut bitmap_info,
        DIB_RGB_COLORS,
      );
      if lines == 0 {
        cleanup_icon(icon, info, dc);
        return None;
      }
      for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
      }
      if pixels.chunks_exact(4).all(|pixel| pixel[3] == 0) {
        for pixel in pixels.chunks_exact_mut(4) {
          pixel[3] = 255;
        }
      }
      cleanup_icon(icon, info, dc);
      ImageBuffer::<Rgba<u8>, _>::from_raw(bitmap.bmWidth as u32, bitmap.bmHeight as u32, pixels)?
        .save(&path)
        .ok()?;
      Some(path)
    }
  }

  pub fn app_identity(pid: u32) -> Option<String> {
    Some(process_image_path(pid)?.to_string_lossy().to_lowercase())
  }

  unsafe fn cleanup_icon(
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    info: ICONINFO,
    dc: windows::Win32::Graphics::Gdi::HDC,
  ) {
    unsafe {
      let _ = DeleteObject(info.hbmColor.into());
      let _ = DeleteObject(info.hbmMask.into());
      let _ = DestroyIcon(icon);
      let _ = DeleteDC(dc);
    }
  }

  pub fn resize_window(
    id: u32,
    pid: u32,
    title: &str,
    width: u32,
    height: u32,
  ) -> Result<(), String> {
    let window = find_window(id, pid, title)?;
    unsafe {
      SetWindowPos(
        window,
        None,
        0,
        0,
        width as i32,
        height as i32,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
      )
      .map_err(|error| error.to_string())
    }
  }
}

#[cfg(target_os = "windows")]
pub use windows_platform::{app_icon, app_identity, resize_window, selectable_window_ids};
