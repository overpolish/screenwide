// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loading the library into another process, starting one of its exports
//! there, and unloading it. Kernel32 sits at one address in every process of
//! a session, so `LoadLibraryW` and `FreeLibrary` are started where they are
//! in Screenwide; an export of the library is found by its offset from the
//! library's start, read from a copy mapped here without running it, from
//! where the library was loaded there.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{s, w, PCSTR, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, HMODULE, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
  CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE,
  TH32CS_SNAPMODULE32,
};
use windows::Win32::System::LibraryLoader::{
  GetModuleHandleW, GetProcAddress, LoadLibraryExW, DONT_RESOLVE_DLL_REFERENCES,
};
use windows::Win32::System::Memory::{
  VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
  CreateRemoteThread, GetExitCodeThread, WaitForSingleObject, LPTHREAD_START_ROUTINE,
  PROCESS_ACCESS_RIGHTS, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE,
  PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

use crate::processes::Process;

/// What loading the library and starting its exports takes of a process.
pub(crate) const RIGHTS: PROCESS_ACCESS_RIGHTS = PROCESS_ACCESS_RIGHTS(
  PROCESS_CREATE_THREAD.0
    | PROCESS_QUERY_INFORMATION.0
    | PROCESS_VM_OPERATION.0
    | PROCESS_VM_READ.0
    | PROCESS_VM_WRITE.0
    | PROCESS_SYNCHRONIZE.0,
);

/// How long a thread started in the app is waited for. Hiding asks each
/// window once, so a slow app is not stalled for long.
const WAIT_MS: u32 = 5_000;

#[derive(Clone, Copy)]
pub(crate) enum Export {
  Hide,
  Restore,
}

impl Export {
  const fn name(self) -> PCSTR {
    match self {
      Self::Hide => s!("screenwide_hide"),
      Self::Restore => s!("screenwide_restore"),
    }
  }
}

type Failure = &'static str;

fn wide(path: &Path) -> Vec<u16> {
  path.as_os_str().encode_wide().chain(Some(0)).collect()
}

/// Starts `start` in `process` with `argument` and waits for it, returning
/// what it returned.
fn run(
  process: &Process,
  start: *const core::ffi::c_void,
  argument: Option<*const core::ffi::c_void>,
) -> Result<u32, Failure> {
  // Safety: `start` is the address of a function taking one pointer and
  // returning a u32, in `process`'s own address space.
  let start: LPTHREAD_START_ROUTINE = unsafe { std::mem::transmute(start) };
  let thread = unsafe { CreateRemoteThread(process.0, None, 0, start, argument, 0, None) }
    .map_err(|_| "the thread could not be started")?;
  let thread = Process(thread);
  if unsafe { WaitForSingleObject(thread.0, WAIT_MS) } != WAIT_OBJECT_0 {
    return Err("the thread did not finish");
  }
  let mut code = 0;
  unsafe { GetExitCodeThread(thread.0, &raw mut code) }.map_err(|_| "the thread said nothing")?;
  Ok(code)
}

fn kernel32(name: PCSTR) -> Result<*const core::ffi::c_void, Failure> {
  let kernel = unsafe { GetModuleHandleW(w!("kernel32.dll")) }.map_err(|_| "no kernel32")?;
  unsafe { GetProcAddress(kernel, name) }
    .map(|function| function as *const core::ffi::c_void)
    .ok_or("kernel32 lacks the function")
}

/// Where the library starts in `pid`, if it is loaded there.
fn loaded_at(pid: u32, library: &Path) -> Option<usize> {
  let wanted = library.to_string_lossy().to_lowercase();
  let snapshot = Process(
    unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }.ok()?,
  );
  let mut entry = MODULEENTRY32W {
    dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
    ..Default::default()
  };
  let mut found = unsafe { Module32FirstW(snapshot.0, &raw mut entry) }.is_ok();
  while found {
    let length = entry
      .szExePath
      .iter()
      .position(|unit| *unit == 0)
      .unwrap_or(0);
    if String::from_utf16_lossy(&entry.szExePath[..length]).to_lowercase() == wanted {
      return Some(entry.modBaseAddr as usize);
    }
    found = unsafe { Module32NextW(snapshot.0, &raw mut entry) }.is_ok();
  }
  None
}

/// Where `export` sits from the library's start, read from a copy mapped
/// here with nothing in it run.
fn offset_of(library: &Path, export: Export) -> Result<usize, Failure> {
  let path = wide(library);
  let module = unsafe { LoadLibraryExW(PCWSTR(path.as_ptr()), None, DONT_RESOLVE_DLL_REFERENCES) }
    .map_err(|_| "the library could not be read")?;
  let function = unsafe { GetProcAddress(module, export.name()) };
  let _ = unsafe { FreeLibrary(module) };
  let function = function.ok_or("the library lacks the export")?;
  Ok(function as usize - module.0 as usize)
}

fn load(process: &Process, library: &Path) -> Result<(), Failure> {
  let path = wide(library);
  let size = path.len() * std::mem::size_of::<u16>();
  let remote = unsafe {
    VirtualAllocEx(
      process.0,
      None,
      size,
      MEM_COMMIT | MEM_RESERVE,
      PAGE_READWRITE,
    )
  };
  if remote.is_null() {
    return Err("no room for the library's path");
  }
  let written = unsafe { WriteProcessMemory(process.0, remote, path.as_ptr().cast(), size, None) };
  let loaded = written
    .map_err(|_| "the library's path could not be written")
    .and_then(|()| run(process, kernel32(s!("LoadLibraryW"))?, Some(remote)));
  let _ = unsafe { VirtualFreeEx(process.0, remote, 0, MEM_RELEASE) };
  // LoadLibraryW's handle, cut to 32 bits: only whether it is zero counts.
  match loaded? {
    0 => Err("the library did not load"),
    _ => Ok(()),
  }
}

/// Starts `export` in `pid`, loading the library first where it is not.
pub(crate) fn call(pid: u32, library: &Path, export: Export) -> Result<u32, Failure> {
  let process = Process::open(pid, RIGHTS).ok_or("the process cannot be opened")?;
  if loaded_at(pid, library).is_none() {
    load(&process, library)?;
  }
  let base = loaded_at(pid, library).ok_or("the library is not loaded")?;
  let start = (base + offset_of(library, export)?) as *const core::ffi::c_void;
  run(&process, start, None)
}

/// Unloads the library from `pid`, where it is loaded.
pub(crate) fn unload(pid: u32, library: &Path) -> Result<(), Failure> {
  let Some(base) = loaded_at(pid, library) else {
    return Ok(());
  };
  let process = Process::open(pid, RIGHTS).ok_or("the process cannot be opened")?;
  let module = HMODULE(base as *mut _);
  run(
    &process,
    kernel32(s!("FreeLibrary"))?,
    Some(module.0.cast_const()),
  )
  .map(|_| ())
}
