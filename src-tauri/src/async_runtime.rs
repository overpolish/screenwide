// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The async runtime Tauri runs commands and their replies on, built so
//! autoreleased objects do not outlive the work that made them.
//!
//! WebKit autoreleases a webview's views and window while a command's reply
//! is handed back on a worker. A thread with no pool of its own keeps such
//! objects until it exits, and Tokio's workers live as long as the app, so
//! every closed editor stayed in memory with its layers and surfaces. Each
//! worker empties a pool whenever it goes idle. Blocking threads have no such
//! moment, so they empty theirs as they exit, which they do soon after their
//! last task.

use std::{cell::Cell, ffi::c_void, sync::OnceLock};

use objc2::ffi::{objc_autoreleasePoolPop, objc_autoreleasePoolPush};

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

thread_local! {
  static POOL: Cell<*mut c_void> = const { Cell::new(std::ptr::null_mut()) };
}

/// Has Tauri run its async work on this runtime. Must run before the app is
/// built, which is when Tauri would start its own.
pub fn install() {
  let runtime = RUNTIME.get_or_init(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      // Tokio's default of ten seconds kept each closed editor whole for as
      // long, and a burst of screenshots held every editor at once.
      .thread_keep_alive(std::time::Duration::from_secs(1))
      .on_thread_start(open_pool)
      .on_thread_park(drain_pool)
      .on_thread_stop(close_pool)
      .build()
      .expect("the async runtime starts")
  });
  tauri::async_runtime::set(runtime.handle().clone());
}

fn open_pool() {
  POOL.set(unsafe { objc_autoreleasePoolPush() });
}

// A worker parks between tasks, never inside one, so no pool a task opened
// can still be open above this one.
fn drain_pool() {
  let pool = POOL.get();
  if pool.is_null() {
    return;
  }
  unsafe { objc_autoreleasePoolPop(pool) };
  open_pool();
}

fn close_pool() {
  let pool = POOL.replace(std::ptr::null_mut());
  if !pool.is_null() {
    unsafe { objc_autoreleasePoolPop(pool) };
  }
}
