// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows plumbing every DirectComposition overlay is built from.
//!
//! A transparent always-on-top overlay is the same pieces whatever it draws:
//! one `WS_EX_NOREDIRECTIONBITMAP` child window per host whose pixels
//! DirectComposition owns, and one wgpu [`GpuSurface`] per child on the
//! shared device. The pipeline and the scene drawn through them belong to
//! the feature.

mod gpu_surface;
mod window;

pub(crate) use gpu_surface::GpuSurface;
pub(crate) use window::{
  create_child, create_on_owning_thread, disable_transitions, register_class, set_capture_affinity,
};

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED},
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::{
      CreateWindowExW, GetWindowThreadProcessId, LoadCursorW, RegisterClassW,
      SetWindowDisplayAffinity, HMENU, IDC_ARROW, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WNDCLASSW,
      WNDCLASS_STYLES, WNDPROC, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP,
    },
  },
};
