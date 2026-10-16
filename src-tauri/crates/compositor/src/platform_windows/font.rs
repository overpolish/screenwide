// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bundled Inter face registered with GDI for the keyboard artwork. The
//! bytes are the ones the annotations' DirectWrite type is loaded from.

use std::{ffi::c_void, sync::OnceLock};

use screenwide_text::engine::type_device::INTER;
use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

pub fn register_inter_font() {
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
