// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows cursors a recorded pointer is drawn with, from the user's
//! cursor scheme. Windows has seven standard shapes for the sixteen recorded
//! styles; the rest borrow the nearest one (`canonical_style`).

mod scheme;

use std::sync::OnceLock;

use image::RgbaImage;
use windows::{
  core::PCWSTR,
  Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
  },
  Win32::UI::WindowsAndMessaging::{
    DestroyCursor, DrawIconEx, GetIconInfo, GetSystemMetrics, LoadImageW, DI_NORMAL, HCURSOR,
    ICONINFO, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_NO, IDC_SIZENS, IDC_SIZEWE,
    IMAGE_CURSOR, LR_LOADFROMFILE, LR_SHARED, SM_CXCURSOR, SM_CYCURSOR,
  },
};

use screenwide_records::cursor::CursorStyle;

/// One system cursor's bitmap plus the hotspot that bitmap addresses, in its
/// texels, and the size the system draws it at.
pub(super) struct StyleArtwork {
  pub hotspot_x: f64,
  pub hotspot_y: f64,
  pub image: RgbaImage,
  pub points: (f64, f64),
}

struct SystemArtwork(Vec<(CursorStyle, StyleArtwork)>);

/// Loaded by `initialize` at startup rather than on first use, as on macOS: a
/// process that never loads it, as the tests, draws the fallback artwork.
static ARTWORK: OnceLock<Result<SystemArtwork, String>> = OnceLock::new();

/// Cursor effects can enlarge the pointer to 500%. Asking user32 for a large
/// cursor makes it pick the sharpest image the cursor resource carries rather
/// than the nominal 32 px one.
const ARTWORK_SIZE: u32 = 128;

pub(super) fn initialize() {
  if let Err(error) = ARTWORK.get_or_init(load) {
    eprintln!("Could not load Windows cursor artwork: {error}");
  }
}

pub(super) fn style_artwork(style: CursorStyle) -> Option<&'static StyleArtwork> {
  let style = canonical_style(style);
  ARTWORK
    .get()
    .and_then(|result| result.as_ref().ok())
    .and_then(|artwork| {
      artwork
        .0
        .iter()
        .find_map(|(candidate, entry)| (*candidate == style).then_some(entry))
    })
}

pub(super) fn artwork(style: CursorStyle) -> Option<&'static RgbaImage> {
  style_artwork(style).map(|entry| &entry.image)
}

/// The standard shape a style is drawn with.
pub(super) fn canonical_style(style: CursorStyle) -> CursorStyle {
  match style {
    CursorStyle::VerticalIBeam => CursorStyle::IBeam,
    CursorStyle::ClosedHand | CursorStyle::OpenHand => CursorStyle::PointingHand,
    CursorStyle::Custom
    | CursorStyle::ContextMenu
    | CursorStyle::DisappearingItem
    | CursorStyle::DragCopy
    | CursorStyle::DragLink
    | CursorStyle::ZoomIn
    | CursorStyle::ZoomOut => CursorStyle::Arrow,
    style => style,
  }
}

fn load() -> Result<SystemArtwork, String> {
  let cursors = [
    (CursorStyle::Arrow, IDC_ARROW, "Arrow", "aero_arrow.cur"),
    (CursorStyle::IBeam, IDC_IBEAM, "IBeam", "beam_r.cur"),
    (
      CursorStyle::ResizeHorizontal,
      IDC_SIZEWE,
      "SizeWE",
      "aero_ew.cur",
    ),
    (
      CursorStyle::ResizeVertical,
      IDC_SIZENS,
      "SizeNS",
      "aero_ns.cur",
    ),
    (CursorStyle::PointingHand, IDC_HAND, "Hand", "aero_link.cur"),
    (
      CursorStyle::Crosshair,
      IDC_CROSS,
      "Crosshair",
      "cross_r.cur",
    ),
    (CursorStyle::NotAllowed, IDC_NO, "No", "aero_unavail.cur"),
  ];
  let points = unsafe { (GetSystemMetrics(SM_CXCURSOR), GetSystemMetrics(SM_CYCURSOR)) };
  let points = (f64::from(points.0.max(1)), f64::from(points.1.max(1)));
  cursors
    .into_iter()
    .map(|(style, cursor, scheme_value, fallback_file)| {
      let (hotspot, image) = cursor_artwork(cursor, scheme_value, fallback_file)
        .map_err(|error| format!("Could not draw the {style:?} system cursor: {error}"))?;
      Ok((
        style,
        StyleArtwork {
          hotspot_x: hotspot.0,
          hotspot_y: hotspot.1,
          image,
          points,
        },
      ))
    })
    .collect::<Result<Vec<_>, String>>()
    .map(SystemArtwork)
}

/// The cursor the scheme names for `scheme_value`, or the standard one, as
/// straight RGBA and its hotspot in texels.
fn cursor_artwork(
  standard: PCWSTR,
  scheme_value: &str,
  fallback_file: &str,
) -> Result<((f64, f64), RgbaImage), String> {
  let size = ARTWORK_SIZE as i32;
  let scheme_path = scheme::cursor_path(scheme_value, fallback_file);
  let file_cursor = scheme_path.as_ref().and_then(|path| {
    unsafe {
      LoadImageW(
        None,
        PCWSTR(path.as_ptr()),
        IMAGE_CURSOR,
        size,
        size,
        LR_LOADFROMFILE,
      )
    }
    .ok()
  });
  let (cursor, owned) = match file_cursor {
    Some(cursor) => (HCURSOR(cursor.0), true),
    None => {
      let cursor = unsafe { LoadImageW(None, standard, IMAGE_CURSOR, size, size, LR_SHARED) }
        .map_err(|error| error.to_string())?;
      (HCURSOR(cursor.0), false)
    }
  };
  let artwork = hotspot(cursor).and_then(|hotspot| {
    let black = render(cursor, 0)?;
    let white = render(cursor, 255)?;
    Ok((hotspot, unmatted(&black, &white)))
  });
  if owned {
    let _ = unsafe { DestroyCursor(cursor) };
  }
  let (hotspot, rgba) = artwork?;
  let image = RgbaImage::from_raw(ARTWORK_SIZE, ARTWORK_SIZE, rgba)
    .ok_or_else(|| "The cursor bitmap does not fill its size".to_owned())?;
  Ok((hotspot, image))
}

fn hotspot(cursor: HCURSOR) -> Result<(f64, f64), String> {
  let mut info = ICONINFO::default();
  unsafe { GetIconInfo(cursor.into(), &mut info) }.map_err(|error| error.to_string())?;
  if !info.hbmColor.is_invalid() {
    let _ = unsafe { DeleteObject(info.hbmColor.into()) };
  }
  if !info.hbmMask.is_invalid() {
    let _ = unsafe { DeleteObject(info.hbmMask.into()) };
  }
  Ok((f64::from(info.xHotspot), f64::from(info.yHotspot)))
}

/// The cursor drawn over an opaque `background` grey, as BGRA.
fn render(cursor: HCURSOR, background: u8) -> Result<Vec<u8>, String> {
  let size = ARTWORK_SIZE as i32;
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: size,
      biHeight: -size,
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
  let length = (ARTWORK_SIZE * ARTWORK_SIZE * 4) as usize;
  let pixels = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
  for pixel in pixels.chunks_exact_mut(4) {
    pixel.fill(background);
    pixel[3] = 255;
  }
  let drawn = unsafe { DrawIconEx(dc, 0, 0, cursor.into(), size, size, 0, None, DI_NORMAL) };
  let result = drawn
    .map(|()| pixels.to_vec())
    .map_err(|error| error.to_string());
  unsafe {
    SelectObject(dc, old);
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(dc);
  }
  result
}

/// GDI draws a cursor without alpha for some schemes (inverting and masked
/// cursors), so its coverage is recovered from the cursor drawn over black
/// and over white: the background shows through by the difference.
fn unmatted(black: &[u8], white: &[u8]) -> Vec<u8> {
  let mut rgba = vec![0_u8; black.len()];
  for ((output, black), white) in rgba
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
      // BGRA in, RGBA out.
      for (target, source) in [(0, 2), (1, 1), (2, 0)] {
        output[target] = ((u32::from(black[source]) * 255 + u32::from(alpha) / 2)
          / u32::from(alpha))
        .min(255) as u8;
      }
    }
  }
  rgba
}
