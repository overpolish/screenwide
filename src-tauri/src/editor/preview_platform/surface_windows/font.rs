// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bundled Inter face: registered with GDI for the keyboard artwork, and
//! loaded by DirectWrite for the annotations' type.

use std::{ffi::c_void, sync::OnceLock};

use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

pub(in crate::editor::preview_platform::surface) static INTER: &[u8] =
  include_bytes!("../../../../assets/Inter-VariableFont_opsz,wght.ttf");

pub(in crate::editor::preview_platform::surface) fn register_inter_font() {
  static REGISTERED: OnceLock<()> = OnceLock::new();
  REGISTERED.get_or_init(|| {
    let mut fonts = 0u32;
    let _ = unsafe {
      AddFontMemResourceEx(
        INTER.as_ptr().cast::<c_void>(),
        INTER.len() as u32,
        None,
        &raw mut fonts,
      )
    };
  });
}
