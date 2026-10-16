// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The families installed on Windows, read from DirectWrite's system
//! collection: each family's name, the weights it has upright, and whether
//! it has an italic.

use windows::Win32::Graphics::DirectWrite::{
  DWriteCreateFactory, IDWriteFactory6, IDWriteLocalizedStrings, DWRITE_FACTORY_TYPE_SHARED,
  DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC, DWRITE_FONT_STYLE_NORMAL,
};

use crate::engine::installed::InstalledFamily;

/// The English name where the family has one, otherwise its first.
fn family_name(names: &IDWriteLocalizedStrings) -> windows::core::Result<String> {
  unsafe {
    let mut index = 0_u32;
    let mut exists = windows::core::BOOL::default();
    names.FindLocaleName(windows::core::w!("en-us"), &mut index, &mut exists)?;
    if !exists.as_bool() {
      index = 0;
    }
    let length = names.GetStringLength(index)? as usize;
    let mut name = vec![0_u16; length + 1];
    names.GetString(index, &mut name)?;
    Ok(String::from_utf16_lossy(&name[..length]))
  }
}

pub(crate) fn installed_families() -> windows::core::Result<Vec<InstalledFamily>> {
  unsafe {
    let factory: IDWriteFactory6 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
    let collection =
      factory.GetSystemFontCollection(false, DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC)?;
    let mut families = Vec::new();
    for index in 0..collection.GetFontFamilyCount() {
      let family = collection.GetFontFamily(index)?;
      let name = family_name(&family.GetFamilyNames()?)?;
      // Families whose names start with an at sign are vertical twins of
      // others, for East Asian text set top to bottom.
      if name.starts_with('@') {
        continue;
      }
      let mut weights = Vec::new();
      let mut italic = false;
      for font_index in 0..family.GetFontCount() {
        let font = family.GetFont(font_index)?;
        if font.GetStyle() == DWRITE_FONT_STYLE_NORMAL {
          weights.push(font.GetWeight().0.clamp(100, 900) as u16);
        } else {
          italic = true;
        }
      }
      if !weights.is_empty() {
        families.push(InstalledFamily {
          name,
          weights,
          italic,
        });
      }
    }
    Ok(families)
  }
}
