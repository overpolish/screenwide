// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use cidre::cg::EventSrcState;

pub(super) fn key_down(code: u16) -> bool {
  EventSrcState::HidSys.key_state(code)
}

pub(super) fn button_down(button: u32) -> bool {
  unsafe extern "C" {
    fn CGEventSourceButtonState(state: EventSrcState, button: u32) -> bool;
  }
  unsafe { CGEventSourceButtonState(EventSrcState::HidSys, button) }
}
