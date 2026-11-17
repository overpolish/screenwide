// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A captured HWND as the shared Glide tween drives it, in physical pixels.

use windows::{
  core::PWSTR,
  Win32::{
    Foundation::CloseHandle,
    System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
      PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
  },
};

use super::target::WindowTarget;
use crate::glide::core::{tween::GlideWindow, GlideFrame};

pub(super) use crate::glide::core::tween::{animate_to, cancel, start};

impl GlideWindow for WindowTarget {
  fn duplicate(&self) -> Self {
    *self
  }

  fn is_movable(&self) -> bool {
    WindowTarget::is_movable(*self)
  }

  fn is_resizable(&self) -> bool {
    WindowTarget::is_resizable(*self)
  }

  fn frame(&self) -> Option<GlideFrame> {
    WindowTarget::frame(*self).ok()
  }

  fn min_size(&self) -> Option<(f64, f64)> {
    WindowTarget::min_size(*self)
  }

  fn prepare_for_move(&mut self) -> Option<GlideFrame> {
    WindowTarget::prepare_for_move(*self)
      .map_err(|error| eprintln!("Could not move the Glide window: {error}"))
      .ok()
  }

  /// `SetWindowPos` writes the origin and size together, so the settling
  /// frame needs no second write.
  fn place(&mut self, frame: GlideFrame, resizes: bool, _settling: bool) -> bool {
    if resizes {
      self.set_frame(frame).is_ok()
    } else {
      self.set_origin(frame).is_ok()
    }
  }

  fn move_to(&mut self, frame: GlideFrame) -> bool {
    self.set_origin(frame).is_ok()
  }

  fn raise(&self) {
    WindowTarget::raise(*self);
  }

  fn minimize(&mut self) {
    WindowTarget::minimize(*self);
  }

  fn restore(&self, _original: GlideFrame) {
    WindowTarget::restore(*self);
  }

  fn app_name(&self) -> Option<String> {
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(self.native_hwnd(), Some(&mut process_id)) };
    let image = unsafe {
      let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
      let mut buffer = [0_u16; 1024];
      let mut length = buffer.len() as u32;
      let result = QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_WIN32,
        PWSTR::from_raw(buffer.as_mut_ptr()),
        &mut length,
      );
      let _ = CloseHandle(process);
      result.ok()?;
      String::from_utf16_lossy(&buffer[..length as usize])
    };
    image.rsplit('\\').next().map(str::to_owned)
  }
}
