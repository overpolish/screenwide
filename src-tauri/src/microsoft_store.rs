// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether the app runs from its Microsoft Store package (MSIX, laid out by
//! `scripts/package-msix.mjs` from `msix/AppxManifest.xml`), and the startup
//! task that package launches it at login with. The Store installs and
//! updates the package, and Windows redirects a packaged app's registry
//! writes, so a packaged app neither updates itself nor registers at login
//! through the autostart plugin.

/// Whether the process runs with a package identity. Never on macOS; on
/// Windows, only when installed from the Store package or registered from
/// its folder.
pub fn is_packaged() -> bool {
  #[cfg(target_os = "windows")]
  {
    use std::sync::OnceLock;

    use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    static PACKAGED: OnceLock<bool> = OnceLock::new();
    *PACKAGED.get_or_init(|| {
      let mut length = 0u32;
      // Asked with no buffer, Windows answers with the length the package's
      // name needs, or with "no package" when the process has none.
      let answer = unsafe { GetCurrentPackageFullName(&mut length, None) };
      answer != APPMODEL_ERROR_NO_PACKAGE
    })
  }
  #[cfg(not(target_os = "windows"))]
  false
}

/// The package's cache folder, as the real path other processes can reach:
/// a packaged app's writes under the usual AppData paths are redirected into
/// its package, out of sight of every other process.
#[cfg(target_os = "windows")]
pub fn cache_folder() -> Option<std::path::PathBuf> {
  use windows::Storage::ApplicationData;

  let folder = ApplicationData::Current()
    .and_then(|data| data.LocalCacheFolder())
    .and_then(|folder| folder.Path())
    .ok()?;
  Some(std::path::PathBuf::from(folder.to_os_string()))
}

#[cfg(target_os = "windows")]
pub mod startup_task {
  use windows::core::HSTRING;
  use windows::ApplicationModel::{StartupTask, StartupTaskState};

  /// The `TaskId` the manifest declares the startup task under.
  const TASK_ID: &str = "ScreenwideStartup";

  /// Runs a WinRT call on a thread of its own and waits for it, so the
  /// thread asking never blocks inside an operation that might need it.
  fn off_thread<T: Send + 'static>(
    call: impl FnOnce() -> windows::core::Result<T> + Send + 'static,
  ) -> Result<T, String> {
    std::thread::spawn(call)
      .join()
      .map_err(|_| "The startup task could not be reached".to_owned())?
      .map_err(|error| error.to_string())
  }

  fn task() -> windows::core::Result<StartupTask> {
    StartupTask::GetAsync(&HSTRING::from(TASK_ID))?.join()
  }

  fn launches(state: StartupTaskState) -> bool {
    state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy
  }

  pub fn is_enabled() -> bool {
    off_thread(|| task()?.State()).is_ok_and(launches)
  }

  /// Turns the task on or off and answers whether it now launches the app.
  /// A task the user turned off in Task Manager or Settings stays off until
  /// they turn it back on there; Windows does not let the app override them.
  pub fn set(enabled: bool) -> Result<bool, String> {
    off_thread(move || {
      let task = task()?;
      if enabled {
        task.RequestEnableAsync()?.join()
      } else {
        task.Disable()?;
        task.State()
      }
    })
    .map(launches)
  }
}
