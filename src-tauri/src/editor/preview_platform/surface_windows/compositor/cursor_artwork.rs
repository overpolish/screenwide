// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn cursor_scheme_path(value_name: &str, fallback_name: &str) -> Option<PathBuf> {
  let value_name = value_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let flags = RRF_RT_REG_SZ | RRF_ZEROONFAILURE;
  let mut byte_length = 0;
  if unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("Control Panel\\Cursors"),
      PCWSTR(value_name.as_ptr()),
      flags,
      None,
      None,
      Some(&mut byte_length),
    )
  } != ERROR_SUCCESS
  {
    return None;
  }
  let mut value = vec![0_u16; (byte_length as usize).div_ceil(2).max(1)];
  if unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("Control Panel\\Cursors"),
      PCWSTR(value_name.as_ptr()),
      flags,
      None,
      Some(value.as_mut_ptr().cast()),
      Some(&mut byte_length),
    )
  } != ERROR_SUCCESS
  {
    return None;
  }
  let end = value
    .iter()
    .position(|character| *character == 0)
    .unwrap_or(value.len());
  value.truncate(end);
  if value.is_empty() {
    value = std::env::var_os("WINDIR")?
      .to_string_lossy()
      .encode_utf16()
      .chain("\\Cursors\\".encode_utf16())
      .chain(fallback_name.encode_utf16())
      .collect();
  }
  value.push(0);
  let expanded_length = unsafe { ExpandEnvironmentStringsW(PCWSTR(value.as_ptr()), None) };
  if expanded_length == 0 {
    return None;
  }
  let mut expanded = vec![0_u16; expanded_length as usize];
  let written =
    unsafe { ExpandEnvironmentStringsW(PCWSTR(value.as_ptr()), Some(expanded.as_mut_slice())) };
  if written == 0 || written > expanded_length {
    return None;
  }
  expanded.truncate(written.saturating_sub(1) as usize);
  Some(PathBuf::from(String::from_utf16_lossy(&expanded)))
}

pub(super) fn native_cursor_pixels(
  cursor_name: PCWSTR,
  scheme_value: &str,
  fallback_file: &str,
) -> Result<(u32, u32, [f32; 2], Vec<u8>), String> {
  // Cursor effects can enlarge artwork to 500%. Asking user32 for a large
  // cursor makes it select the highest-resolution image embedded in the
  // active Windows cursor resource instead of permanently baking the atlas
  // from the nominal 32 px representation.
  const ATLAS_CURSOR_SIZE: u32 = 128;
  let width = ATLAS_CURSOR_SIZE;
  let height = ATLAS_CURSOR_SIZE;
  let scheme_path = cursor_scheme_path(scheme_value, fallback_file);
  let scheme_path_wide = scheme_path.as_ref().map(|path| {
    path
      .as_os_str()
      .to_string_lossy()
      .encode_utf16()
      .chain(Some(0))
      .collect::<Vec<_>>()
  });
  let file_cursor = scheme_path_wide.as_ref().and_then(|path| {
    unsafe {
      LoadImageW(
        None,
        PCWSTR(path.as_ptr()),
        IMAGE_CURSOR,
        width as i32,
        height as i32,
        LR_LOADFROMFILE,
      )
    }
    .ok()
  });
  let (cursor, owned) = if let Some(cursor) = file_cursor {
    (HCURSOR(cursor.0), true)
  } else {
    let cursor = unsafe {
      LoadImageW(
        None,
        cursor_name,
        IMAGE_CURSOR,
        width as i32,
        height as i32,
        LR_SHARED,
      )
    }
    .map_err(|error| error.to_string())?;
    (HCURSOR(cursor.0), false)
  };
  let mut cursor_info = windows::Win32::UI::WindowsAndMessaging::ICONINFO::default();
  if let Err(error) = unsafe { GetIconInfo(cursor.into(), &mut cursor_info) } {
    if owned {
      let _ = unsafe { DestroyCursor(cursor) };
    }
    return Err(error.to_string());
  }
  let hotspot = [
    (cursor_info.xHotspot as f32 + 0.5) / width as f32,
    (cursor_info.yHotspot as f32 + 0.5) / height as f32,
  ];
  if !cursor_info.hbmColor.is_invalid() {
    let _ = unsafe { DeleteObject(cursor_info.hbmColor.into()) };
  }
  if !cursor_info.hbmMask.is_invalid() {
    let _ = unsafe { DeleteObject(cursor_info.hbmMask.into()) };
  }
  let render = |background: u8| -> Result<Vec<u8>, String> {
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: -(height as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return Err("Windows could not create a cursor drawing context".to_owned());
    }
    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe {
      CreateDIBSection(
        Some(dc),
        &raw const info,
        DIB_RGB_COLORS,
        &mut bits,
        None,
        0,
      )
    }
    .map_err(|error| {
      let _ = unsafe { DeleteDC(dc) };
      error.to_string()
    })?;
    let old = unsafe { SelectObject(dc, bitmap.into()) };
    let length = (width * height * 4) as usize;
    let pixels = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
    for pixel in pixels.chunks_exact_mut(4) {
      pixel.fill(background);
      pixel[3] = 255;
    }
    let drawn = unsafe {
      DrawIconEx(
        dc,
        0,
        0,
        cursor.into(),
        width as i32,
        height as i32,
        0,
        None,
        DI_NORMAL,
      )
    };
    let result = drawn
      .map(|()| pixels.to_vec())
      .map_err(|error| error.to_string());
    unsafe {
      SelectObject(dc, old);
      let _ = DeleteObject(bitmap.into());
      let _ = DeleteDC(dc);
    }
    result
  };
  let black = render(0);
  let white = render(255);
  if owned {
    let _ = unsafe { DestroyCursor(cursor) };
  }
  let black = black?;
  let white = white?;
  let mut pixels = vec![0_u8; black.len()];
  for ((output, black), white) in pixels
    .chunks_exact_mut(4)
    .zip(black.chunks_exact(4))
    .zip(white.chunks_exact(4))
  {
    let background = (0..3)
      .map(|channel| i32::from(white[channel]) - i32::from(black[channel]))
      .sum::<i32>()
      / 3;
    let alpha = (255 - background).clamp(0, 255) as u8;
    output[3] = alpha;
    if alpha != 0 {
      for channel in 0..3 {
        output[channel] = ((u32::from(black[channel]) * 255 + u32::from(alpha) / 2)
          / u32::from(alpha))
        .min(255) as u8;
      }
    }
  }
  Ok((width, height, hotspot, pixels))
}
