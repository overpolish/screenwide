// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Registers the bundled Inter face with GDI for the native preview surfaces
//! that rasterise their own text.

use std::{ffi::c_void, sync::OnceLock};

use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

pub(in crate::editor::preview_platform::surface) fn register_inter_font() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    static FONT: &[u8] = include_bytes!("../../../../assets/Inter-VariableFont_opsz,wght.ttf");
    let mut fonts = 0u32;
    let _ = unsafe {
      AddFontMemResourceEx(
        FONT.as_ptr().cast::<c_void>(),
        FONT.len() as u32,
        None,
        &raw mut fonts,
      )
    };
  });
}
