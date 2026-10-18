// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keyboard as the Windows host reads it: the modifiers held now and the
//! low-level hook an overlay captures the keyboard through.

use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use super::super::region_osc::host::KeyboardOwner;
use super::WindowsRegionHost;
use crate::osc::keyboard_windows::Overlay;

fn down(key: u16) -> bool {
  (unsafe { GetKeyState(i32::from(key)) }) < 0
}

impl WindowsRegionHost {
  /// The keyboard modifiers held now: bit 0 Super, 1 Ctrl, 2 Alt, 3 Shift.
  pub(in super::super) fn keyboard_modifiers(&self) -> u32 {
    let super_key = down(VK_LWIN.0) || down(VK_RWIN.0);
    (u32::from(down(VK_CONTROL.0)) * 2)
      | (u32::from(down(VK_SHIFT.0)) * 8)
      | (u32::from(down(VK_MENU.0)) * 4)
      | u32::from(super_key)
  }

  pub(in super::super) fn alt_pressed(&self) -> bool {
    crate::ruler::windows_alt_pressed()
  }
}

pub(super) fn overlay(owner: KeyboardOwner) -> Overlay {
  match owner {
    KeyboardOwner::Ruler => Overlay::Ruler,
    KeyboardOwner::TextRecognition => Overlay::TextRecognition,
  }
}
