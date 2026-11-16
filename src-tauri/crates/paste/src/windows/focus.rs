// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What has the keyboard, as UI Automation tells it: a password field, which
//! nothing typed for may pass through the clipboard, and otherwise whether
//! it takes text. Anything with a value says so by whether the value can be
//! set; without one, an edit box, a document, a combo box or anything with
//! text to read does, as a terminal or an editable web element; a list of
//! files does not. An app that does not answer is taken to have pasted, as
//! it most often has.

use windows::Win32::System::Com::{
  CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
  CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationValuePattern,
  UIA_ComboBoxControlTypeId, UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_TextPatternId,
  UIA_ValuePatternId,
};

use crate::Focus;

/// What the element with the keyboard was seen to be.
pub(super) enum Seen {
  Password,
  Focus(Focus),
}

pub(super) fn seen() -> Seen {
  // Safety: COM is set up for this thread and released before returning;
  // the automation object and the element live inside it.
  unsafe {
    let initialized = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
    let seen = CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
      .and_then(|automation| automation.GetFocusedElement())
      .map_or(Seen::Focus(Focus::Unknown), |element| classify(&element));
    if initialized {
      CoUninitialize();
    }
    seen
  }
}

/// # Safety
///
/// COM must be set up on this thread.
unsafe fn classify(element: &IUIAutomationElement) -> Seen {
  unsafe {
    if element
      .CurrentIsPassword()
      .is_ok_and(|password| password.as_bool())
    {
      return Seen::Password;
    }
    // A value said to be read-only settles it: a web page's own document,
    // which Chromium and WebView2 report as a read-only document with text
    // to read whenever no field on it has the keyboard, or a field that is
    // read-only. Notepad's editor and every field typed into say writable.
    let read_only = element
      .GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
      .and_then(|value| value.CurrentIsReadOnly())
      .map(|read_only| read_only.as_bool());
    if let Ok(read_only) = read_only {
      return Seen::Focus(if read_only {
        Focus::Elsewhere
      } else {
        Focus::TakesText
      });
    }
    let text_control = element.CurrentControlType().is_ok_and(|kind| {
      [
        UIA_EditControlTypeId,
        UIA_DocumentControlTypeId,
        UIA_ComboBoxControlTypeId,
      ]
      .contains(&kind)
    });
    let text = element.GetCurrentPattern(UIA_TextPatternId).is_ok();
    Seen::Focus(if text_control || text {
      Focus::TakesText
    } else {
      Focus::Elsewhere
    })
  }
}
