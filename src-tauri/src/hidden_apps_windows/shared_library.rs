// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window hider for a Store install. Every file in the package's install
//! folder may be executed only by the package's own processes, so the apps
//! being hidden cannot load the library from there. A copy is kept in the
//! package's cache folder instead, under one name across runs: the restore
//! after a Screenwide that stopped mid-recording finds the library in each
//! hidden app by the path it was loaded from.

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Authorization::{
  GetNamedSecurityInfoW, SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, GRANT_ACCESS,
  NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_W,
};
use windows::Win32::Security::{
  CreateWellKnownSid, WinBuiltinAnyPackageSid, ACL, DACL_SECURITY_INFORMATION, NO_INHERITANCE,
  PSECURITY_DESCRIPTOR, PSID, SECURITY_MAX_SID_SIZE,
};
use windows::Win32::Storage::FileSystem::{FILE_GENERIC_EXECUTE, FILE_GENERIC_READ};

/// The copy to load, made or brought up to date from `bundled`.
pub(super) fn prepare(bundled: &Path) -> Option<PathBuf> {
  let copy = crate::microsoft_store::cache_folder()?.join(super::LIBRARY);
  let library = std::fs::read(bundled).ok()?;
  if std::fs::read(&copy).ok().as_deref() == Some(library.as_slice()) {
    return Some(copy);
  }
  // An app still holding an older copy, left hidden by a Screenwide that
  // stopped mid-recording, keeps it from being replaced. That copy is then
  // the one to restore the app with, and serves for hiding too.
  match replace(&copy, &library) {
    Ok(()) => Some(copy),
    Err(_) => copy.is_file().then_some(copy),
  }
}

/// Writes the library beside `copy`, lets packaged apps run it as they may
/// from Program Files, then moves it into place in one step.
fn replace(copy: &Path, library: &[u8]) -> std::io::Result<()> {
  let staged = copy.with_extension("dll.new");
  std::fs::write(&staged, library)?;
  let moved = grant_app_packages(&staged).and_then(|()| std::fs::rename(&staged, copy));
  if moved.is_err() {
    let _ = std::fs::remove_file(&staged);
  }
  moved
}

/// Adds read and execute for ALL APPLICATION PACKAGES, which Program Files
/// grants and the cache folder does not, so an app that runs in a sandbox
/// can still load the library.
fn grant_app_packages(path: &Path) -> std::io::Result<()> {
  let error = |code: u32| std::io::Error::from_raw_os_error(code as i32);
  let path: Vec<u16> = path
    .as_os_str()
    .encode_wide()
    .chain(std::iter::once(0))
    .collect();
  let mut sid = [0u8; SECURITY_MAX_SID_SIZE as usize];
  let mut sid_size = sid.len() as u32;
  let sid = PSID(sid.as_mut_ptr().cast());
  unsafe { CreateWellKnownSid(WinBuiltinAnyPackageSid, None, Some(sid), &mut sid_size) }
    .map_err(|failure| error(failure.code().0 as u32))?;

  let mut current: *mut ACL = std::ptr::null_mut();
  let mut descriptor = PSECURITY_DESCRIPTOR::default();
  let read = unsafe {
    GetNamedSecurityInfoW(
      PCWSTR(path.as_ptr()),
      SE_FILE_OBJECT,
      DACL_SECURITY_INFORMATION,
      None,
      None,
      Some(&mut current),
      None,
      &mut descriptor,
    )
  };
  if read.is_err() {
    return Err(error(read.0));
  }
  let entry = EXPLICIT_ACCESS_W {
    grfAccessPermissions: FILE_GENERIC_READ.0 | FILE_GENERIC_EXECUTE.0,
    grfAccessMode: GRANT_ACCESS,
    grfInheritance: NO_INHERITANCE,
    Trustee: TRUSTEE_W {
      MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
      TrusteeForm: TRUSTEE_IS_SID,
      TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
      ptstrName: windows::core::PWSTR(sid.0.cast()),
      ..Default::default()
    },
  };
  let mut granted: *mut ACL = std::ptr::null_mut();
  let merged = unsafe { SetEntriesInAclW(Some(&[entry]), Some(current), &mut granted) };
  let written = if merged.is_ok() {
    unsafe {
      SetNamedSecurityInfoW(
        PCWSTR(path.as_ptr()),
        SE_FILE_OBJECT,
        DACL_SECURITY_INFORMATION,
        None,
        None,
        Some(granted),
        None,
      )
    }
  } else {
    merged
  };
  unsafe {
    let _ = LocalFree(Some(HLOCAL(granted.cast())));
    let _ = LocalFree(Some(HLOCAL(descriptor.0)));
  }
  if written.is_err() {
    return Err(error(written.0));
  }
  Ok(())
}
