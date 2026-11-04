// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is asked of a process before anything is loaded into it: what it
//! runs, when it started, the windows it owns, and whether Screenwide may
//! load the library into it at all.

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, HWND, LPARAM};
use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE;
use windows::Win32::System::Threading::{
  GetCurrentProcess, GetCurrentProcessId, GetProcessInformation, GetProcessTimes, IsWow64Process2,
  OpenProcess, ProcessMachineTypeInfo, QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS,
  PROCESS_MACHINE_INFORMATION, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
  EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
};

use crate::inject::RIGHTS;

/// A process handle closed when dropped.
pub(crate) struct Process(pub(crate) HANDLE);

impl Process {
  pub(crate) fn open(pid: u32, rights: PROCESS_ACCESS_RIGHTS) -> Option<Self> {
    unsafe { OpenProcess(rights, false, pid) }.ok().map(Self)
  }
}

impl Drop for Process {
  fn drop(&mut self) {
    let _ = unsafe { CloseHandle(self.0) };
  }
}

/// The processes that own a visible top-level window, each once: an app's
/// helpers, which own none, are left alone.
pub fn window_owners() -> Vec<u32> {
  unsafe extern "system" fn collect(window: HWND, found: LPARAM) -> BOOL {
    if unsafe { IsWindowVisible(window) }.as_bool() {
      let mut pid = 0;
      unsafe { GetWindowThreadProcessId(window, Some(&raw mut pid)) };
      let found = unsafe { &mut *(found.0 as *mut Vec<u32>) };
      if pid != 0 && !found.contains(&pid) {
        found.push(pid);
      }
    }
    true.into()
  }
  let mut found: Vec<u32> = Vec::new();
  let _ = unsafe { EnumWindows(Some(collect), LPARAM(&raw mut found as isize)) };
  let ours = unsafe { GetCurrentProcessId() };
  found.retain(|pid| *pid != ours);
  found
}

/// The full path a process runs, in lower case, as Settings names apps.
pub fn image_of(pid: u32) -> Option<String> {
  let process = Process::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
  let mut buffer = [0u16; 1024];
  let mut length = buffer.len() as u32;
  unsafe {
    QueryFullProcessImageNameW(
      process.0,
      PROCESS_NAME_WIN32,
      PWSTR(buffer.as_mut_ptr()),
      &raw mut length,
    )
  }
  .ok()?;
  Some(String::from_utf16_lossy(&buffer[..length as usize]).to_lowercase())
}

/// When the process started, in 100 ns units since 1601.
pub(crate) fn started(pid: u32) -> Option<u64> {
  let process = Process::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
  let mut created = FILETIME::default();
  let mut unused = [FILETIME::default(); 3];
  unsafe {
    GetProcessTimes(
      process.0,
      &raw mut created,
      &raw mut unused[0],
      &raw mut unused[1],
      &raw mut unused[2],
    )
  }
  .ok()?;
  Some((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

/// The machine a process's code is built for. Windows 11 says so for
/// emulated processes too; Windows 10 tells 32-bit processes apart only,
/// and the rest are taken for the system's own.
fn machine(process: HANDLE) -> Option<IMAGE_FILE_MACHINE> {
  let mut information = PROCESS_MACHINE_INFORMATION::default();
  let told = unsafe {
    GetProcessInformation(
      process,
      ProcessMachineTypeInfo,
      (&raw mut information).cast(),
      std::mem::size_of::<PROCESS_MACHINE_INFORMATION>() as u32,
    )
  };
  if told.is_ok() {
    return Some(information.ProcessMachine);
  }
  let mut process_machine = IMAGE_FILE_MACHINE::default();
  let mut native = IMAGE_FILE_MACHINE::default();
  unsafe { IsWow64Process2(process, &raw mut process_machine, Some(&raw mut native)) }.ok()?;
  // Unknown: not running under WOW64, so it is the system's own machine.
  Some(if process_machine.0 == 0 {
    native
  } else {
    process_machine
  })
}

/// Whether Screenwide may load the library into `pid`: it opens the process
/// with what loading takes, which an app running as administrator, or a
/// protected one, refuses, and the process runs code for the machine the
/// library is built for, which is Screenwide's own.
pub fn can_hide(pid: u32) -> bool {
  let Some(process) = Process::open(pid, RIGHTS) else {
    return false;
  };
  let ours = machine(unsafe { GetCurrentProcess() });
  ours.is_some() && machine(process.0) == ours
}
