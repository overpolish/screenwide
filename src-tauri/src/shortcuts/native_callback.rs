// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

thread_local! {
  static IN_NATIVE_CALLBACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Runs the body of a native global-shortcut callback.
///
/// The plugin holds its shortcut registry locked for as long as a callback
/// runs, and that mutex is not reentrant: registering or releasing any
/// shortcut from inside one deadlocks the app. Anything a callback reaches
/// that needs the registry asks [`in_native_callback`] and waits for a later
/// turn instead.
pub(crate) fn during_native_callback<T>(work: impl FnOnce() -> T) -> T {
  struct Guard;
  impl Drop for Guard {
    fn drop(&mut self) {
      IN_NATIVE_CALLBACK.set(false);
    }
  }
  IN_NATIVE_CALLBACK.set(true);
  let _guard = Guard;
  work()
}

pub(crate) fn in_native_callback() -> bool {
  IN_NATIVE_CALLBACK.get()
}
