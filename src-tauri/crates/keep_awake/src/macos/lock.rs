// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::NonNull;
use std::sync::Arc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2_foundation::{
  NSDistributedNotificationCenter, NSNotification, NSOperationQueue, NSString,
};

/// Tells whether the screen has locked or unlocked, until dropped.
pub(crate) struct LockWatch(Vec<Retained<ProtocolObject<dyn NSObjectProtocol>>>);

// SAFETY: the notification center is thread-safe, and the tokens are opaque
// handles only ever handed back to it, so the watch may end on any thread.
unsafe impl Send for LockWatch {}

impl LockWatch {
  pub(crate) fn start(changed: impl Fn(bool) + Send + Sync + 'static) -> Result<Self, String> {
    let changed = Arc::new(changed);
    let center = NSDistributedNotificationCenter::defaultCenter();
    // Distributed notifications reach a thread through its run loop; the
    // main thread's always runs, whichever thread starts the watch.
    let queue = NSOperationQueue::mainQueue();
    let observers = [
      ("com.apple.screenIsLocked", true),
      ("com.apple.screenIsUnlocked", false),
    ]
    .into_iter()
    .map(|(name, locked)| {
      let changed = Arc::clone(&changed);
      let block = RcBlock::new(move |_: NonNull<NSNotification>| changed(locked));
      // SAFETY: the block only calls a `Send + Sync` closure, so it may run
      // on the main queue.
      unsafe {
        center.addObserverForName_object_queue_usingBlock(
          Some(&NSString::from_str(name)),
          None,
          Some(&queue),
          &block,
        )
      }
    })
    .collect();
    Ok(Self(observers))
  }
}

impl Drop for LockWatch {
  fn drop(&mut self) {
    let center = NSDistributedNotificationCenter::defaultCenter();
    for observer in &self.0 {
      // SAFETY: each token came from this center and is removed once.
      unsafe { center.removeObserver(observer.as_ref()) };
    }
  }
}
