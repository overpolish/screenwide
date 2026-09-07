// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc,
};

static BUSY: AtomicUsize = AtomicUsize::new(0);

/// Keeps new Glide ownership attempts out until asynchronous native cleanup
/// has completed. Each native tween, cursor landing, and fade owns a lease.
pub(crate) struct BusyLease(Arc<Lease>);

struct Lease;

impl Drop for Lease {
  fn drop(&mut self) {
    BUSY.fetch_sub(1, Ordering::AcqRel);
  }
}

impl BusyLease {
  pub(crate) fn acquire() -> Self {
    BUSY.fetch_add(1, Ordering::AcqRel);
    Self(Arc::new(Lease))
  }

  pub(crate) fn is_busy() -> bool {
    BUSY.load(Ordering::Acquire) != 0
  }
}

impl Clone for BusyLease {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

#[cfg(test)]
mod tests {
  use super::BusyLease;

  #[test]
  fn busy_until_every_lease_is_released() {
    let first = BusyLease::acquire();
    let second = BusyLease::acquire();
    assert!(BusyLease::is_busy());
    drop(first);
    assert!(BusyLease::is_busy());
    drop(second);
    assert!(!BusyLease::is_busy());
  }
}
