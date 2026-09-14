// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows virtual desktop transport. `winvd` uses the Windows 11 24H2
//! VirtualDesktopAccessor ABI and reports an error on older builds, so Glide
//! can keep monitor support available without making desktop movement a hard
//! dependency.

use std::{collections::HashMap, sync::Mutex};

use crate::glide::core::desktops::{Desktop, DesktopAdapter, DesktopGroup, MoveError, Snapshot};
use windows::Win32::Foundation::HWND;

pub(super) struct Adapter {
  hwnd: HWND,
  ids: Mutex<HashMap<String, windows058::core::GUID>>,
}

impl Adapter {
  pub(super) fn new(hwnd: HWND) -> Self {
    Self {
      hwnd,
      ids: Mutex::new(HashMap::new()),
    }
  }
}

impl DesktopAdapter for Adapter {
  fn snapshot(&self) -> Result<Snapshot, MoveError> {
    if !build_supported() {
      return Err(MoveError::Unsupported);
    }
    if winvd::is_pinned_window(legacy(self.hwnd))
      .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?
      || winvd::is_pinned_app(legacy(self.hwnd))
        .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?
    {
      return Err(MoveError::AmbiguousWindow);
    }
    let desktops =
      winvd::get_desktops().map_err(|error| MoveError::Unavailable(format!("{error:?}")))?;
    let membership = winvd::get_desktop_by_window(legacy(self.hwnd))
      .and_then(|desktop| desktop.get_id())
      .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?;
    let current = winvd::get_current_desktop()
      .and_then(|desktop| desktop.get_id())
      .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?;
    let current_id = format!("{current:?}");
    if let Ok(mut ids) = self.ids.lock() {
      ids.clear();
      for desktop in &desktops {
        if let Ok(id) = desktop.get_id() {
          ids.insert(format!("{id:?}"), id);
        }
      }
    }
    Ok(Snapshot {
      membership: vec![format!("{membership:?}")],
      groups: vec![DesktopGroup {
        id: "windows-global".into(),
        current: current_id,
        transitioning: false,
        desktops: desktops
          .into_iter()
          .filter_map(|desktop| {
            desktop.get_id().ok().map(|id| Desktop {
              id: format!("{id:?}"),
              regular: true,
            })
          })
          .collect(),
      }],
    })
  }

  fn carry_window(&self, _group: &str, destination: &str) -> Result<(), MoveError> {
    if !build_supported() {
      return Err(MoveError::Unsupported);
    }
    let desktop = self
      .ids
      .lock()
      .ok()
      .and_then(|ids| ids.get(destination).copied())
      .ok_or(MoveError::AmbiguousWindow)?;
    let source = winvd::get_desktop_by_window(legacy(self.hwnd))
      .and_then(|desktop| desktop.get_id())
      .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?;
    winvd::move_window_to_desktop(desktop, &legacy(self.hwnd))
      .map_err(|error| MoveError::Unavailable(format!("{error:?}")))?;
    if let Err(error) = winvd::switch_desktop(desktop) {
      let _ = winvd::move_window_to_desktop(source, &legacy(self.hwnd));
      return Err(MoveError::Unavailable(format!("{error:?}")));
    }
    Ok(())
  }
}

fn legacy(hwnd: HWND) -> windows058::Win32::Foundation::HWND {
  windows058::Win32::Foundation::HWND(hwnd.0)
}

fn build_supported() -> bool {
  let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
    .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
  else {
    return false;
  };
  let build: u32 = key
    .get_value("CurrentBuildNumber")
    .ok()
    .and_then(|v: String| v.parse().ok())
    .unwrap_or(0);
  let ubr: u32 = key.get_value("UBR").unwrap_or(0);
  build == 26200 || (build == 26100 && ubr >= 2605)
}

pub(super) fn supported() -> bool {
  build_supported()
}

#[cfg(test)]
mod tests {
  #[test]
  #[ignore = "requires Windows 11 24H2 VirtualDesktopAccessor support"]
  fn host_virtual_desktop_api_is_available() {
    assert!(super::supported());
    let desktops = winvd::get_desktops().expect("desktop enumeration");
    assert!(!desktops.is_empty());
    let current = winvd::get_current_desktop().expect("current desktop");
    let id = current.get_id().expect("current desktop GUID");
    eprintln!(
      "Windows Glide virtual desktops: count={}, current={id:?}",
      desktops.len()
    );
  }
}
