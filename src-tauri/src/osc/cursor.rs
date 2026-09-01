// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(
  dead_code,
  reason = "cursor owners are migrated to the shared lease incrementally"
)]

//! Platform-neutral ownership for the one cursor presented by an interactive
//! desktop overlay session.
//!
//! Tools decide the cursor icon, while platform adapters acquire foreground
//! ownership and present it. Lease generations make delayed teardown inert:
//! an old owner can never release or update a newer session's cursor.

use super::protocol::CursorIcon;

#[cfg(target_os = "macos")]
pub(crate) mod macos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorOwner {
  RecordingRegion,
  QuickScreenshot,
  TextRecognition,
  Ruler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorLeaseId {
  owner: CursorOwner,
  generation: u64,
}

impl CursorLeaseId {
  pub const fn owner(self) -> CursorOwner {
    self.owner
  }

  pub const fn generation(self) -> u64 {
    self.generation
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorLease {
  pub id: CursorLeaseId,
  pub icon: CursorIcon,
}

/// The only commands a platform cursor adapter needs to implement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorPresentation {
  Acquire(CursorLease),
  Update(CursorLease),
  Release(CursorLeaseId),
  Transfer {
    from: CursorLeaseId,
    to: CursorLease,
  },
}

/// Separates a portable state change from work required by the OS adapter.
///
/// Releasing an already-suspended lease changes portable state but has no
/// presentation command because that cursor was released during suspension.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorTransition {
  pub state_changed: bool,
  pub presentation: Option<CursorPresentation>,
}

impl CursorTransition {
  const fn ignored() -> Self {
    Self {
      state_changed: false,
      presentation: None,
    }
  }

  const fn presented(presentation: CursorPresentation) -> Self {
    Self {
      state_changed: true,
      presentation: Some(presentation),
    }
  }

  const fn state_only() -> Self {
    Self {
      state_changed: true,
      presentation: None,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorLeaseError {
  AlreadyOwned(CursorLeaseId),
  InvalidIcon,
  GenerationExhausted,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CursorLeaseState {
  generation: u64,
  active: Option<CursorLease>,
  suspended: Vec<CursorLease>,
}

impl CursorLeaseState {
  pub const fn active(&self) -> Option<CursorLease> {
    self.active
  }

  pub fn restorable(&self) -> Option<CursorLease> {
    self.suspended.last().copied()
  }

  pub fn acquire(
    &mut self,
    owner: CursorOwner,
    icon: CursorIcon,
  ) -> Result<(CursorLeaseId, CursorTransition), CursorLeaseError> {
    if icon == CursorIcon::Unchanged {
      return Err(CursorLeaseError::InvalidIcon);
    }
    if let Some(active) = self.active {
      return Err(CursorLeaseError::AlreadyOwned(active.id));
    }
    let lease = self.next_lease(owner, icon)?;
    self.active = Some(lease);
    Ok((
      lease.id,
      CursorTransition::presented(CursorPresentation::Acquire(lease)),
    ))
  }

  /// Atomically replaces the active presentation while retaining the previous
  /// lease for restoration. Platform adapters must not release foreground
  /// ownership between the two cursors.
  pub fn transfer(
    &mut self,
    from: CursorLeaseId,
    owner: CursorOwner,
    icon: CursorIcon,
  ) -> Result<(CursorLeaseId, CursorTransition), CursorLeaseError> {
    if icon == CursorIcon::Unchanged {
      return Err(CursorLeaseError::InvalidIcon);
    }
    let Some(previous) = self.active.filter(|lease| lease.id == from) else {
      return Ok((from, CursorTransition::ignored()));
    };
    let next = self.next_lease(owner, icon)?;
    self.suspended.push(previous);
    self.active = Some(next);
    Ok((
      next.id,
      CursorTransition::presented(CursorPresentation::Transfer { from, to: next }),
    ))
  }

  /// Atomically ends the active lease and restores the most recently
  /// suspended owner. This is the inverse of [`Self::transfer`].
  pub fn restore(&mut self, from: CursorLeaseId, to: CursorLeaseId) -> CursorTransition {
    let Some(current) = self.active.filter(|lease| lease.id == from) else {
      return CursorTransition::ignored();
    };
    let Some(previous) = self
      .suspended
      .last()
      .copied()
      .filter(|lease| lease.id == to)
    else {
      return CursorTransition::ignored();
    };
    self.suspended.pop();
    self.active = Some(previous);
    CursorTransition::presented(CursorPresentation::Transfer {
      from: current.id,
      to: previous,
    })
  }

  pub fn update(&mut self, id: CursorLeaseId, icon: CursorIcon) -> CursorTransition {
    if icon == CursorIcon::Unchanged {
      return CursorTransition::ignored();
    }
    let Some(active) = self.active.as_mut().filter(|lease| lease.id == id) else {
      return CursorTransition::ignored();
    };
    if active.icon == icon {
      return CursorTransition::ignored();
    }
    active.icon = icon;
    CursorTransition::presented(CursorPresentation::Update(*active))
  }

  pub fn suspend(&mut self, id: CursorLeaseId) -> CursorTransition {
    let Some(active) = self.active.filter(|lease| lease.id == id) else {
      return CursorTransition::ignored();
    };
    self.active = None;
    self.suspended.push(active);
    CursorTransition::presented(CursorPresentation::Release(id))
  }

  pub fn resume(&mut self, id: CursorLeaseId) -> CursorTransition {
    if self.active.is_some() {
      return CursorTransition::ignored();
    }
    let Some(index) = self.suspended.iter().position(|lease| lease.id == id) else {
      return CursorTransition::ignored();
    };
    let lease = self.suspended.remove(index);
    self.active = Some(lease);
    CursorTransition::presented(CursorPresentation::Acquire(lease))
  }

  pub fn release(&mut self, id: CursorLeaseId) -> CursorTransition {
    if self.active.is_some_and(|lease| lease.id == id) {
      self.active = None;
      return CursorTransition::presented(CursorPresentation::Release(id));
    }
    let Some(index) = self.suspended.iter().position(|lease| lease.id == id) else {
      return CursorTransition::ignored();
    };
    self.suspended.remove(index);
    CursorTransition::state_only()
  }

  fn next_lease(
    &mut self,
    owner: CursorOwner,
    icon: CursorIcon,
  ) -> Result<CursorLease, CursorLeaseError> {
    self.generation = self
      .generation
      .checked_add(1)
      .ok_or(CursorLeaseError::GenerationExhausted)?;
    Ok(CursorLease {
      id: CursorLeaseId {
        owner,
        generation: self.generation,
      },
      icon,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn acquire(state: &mut CursorLeaseState, owner: CursorOwner, icon: CursorIcon) -> CursorLeaseId {
    state.acquire(owner, icon).unwrap().0
  }

  #[test]
  fn acquire_update_and_release_emit_one_platform_command_each() {
    let mut state = CursorLeaseState::default();
    let (id, acquired) = state
      .acquire(CursorOwner::QuickScreenshot, CursorIcon::Crosshair)
      .unwrap();
    assert_eq!(
      acquired.presentation,
      Some(CursorPresentation::Acquire(CursorLease {
        id,
        icon: CursorIcon::Crosshair,
      }))
    );
    assert_eq!(
      state.update(id, CursorIcon::ClosedHand).presentation,
      Some(CursorPresentation::Update(CursorLease {
        id,
        icon: CursorIcon::ClosedHand,
      }))
    );
    assert_eq!(
      state.release(id).presentation,
      Some(CursorPresentation::Release(id))
    );
    assert_eq!(state.active(), None);
  }

  #[test]
  fn ruler_can_atomically_transfer_ownership_to_quick_screenshot_and_back() {
    let mut state = CursorLeaseState::default();
    let ruler = acquire(&mut state, CursorOwner::Ruler, CursorIcon::Crosshair);
    let (screenshot, transferred) = state
      .transfer(ruler, CursorOwner::QuickScreenshot, CursorIcon::Crosshair)
      .unwrap();
    assert_eq!(
      transferred.presentation,
      Some(CursorPresentation::Transfer {
        from: ruler,
        to: CursorLease {
          id: screenshot,
          icon: CursorIcon::Crosshair,
        },
      })
    );
    assert_eq!(state.active().unwrap().id, screenshot);
    assert_eq!(
      state.restore(screenshot, ruler).presentation,
      Some(CursorPresentation::Transfer {
        from: screenshot,
        to: CursorLease {
          id: ruler,
          icon: CursorIcon::Crosshair,
        },
      })
    );
    assert_eq!(state.active().unwrap().id, ruler);
    assert_eq!(state.release(screenshot), CursorTransition::ignored());
  }

  #[test]
  fn stale_teardown_cannot_release_a_new_generation() {
    let mut state = CursorLeaseState::default();
    let first = acquire(
      &mut state,
      CursorOwner::QuickScreenshot,
      CursorIcon::Crosshair,
    );
    state.release(first);
    let second = acquire(
      &mut state,
      CursorOwner::QuickScreenshot,
      CursorIcon::Crosshair,
    );
    assert!(second.generation() > first.generation());
    assert_eq!(state.release(first), CursorTransition::ignored());
    assert_eq!(state.active().unwrap().id, second);
  }

  #[test]
  fn suspended_teardown_does_not_release_the_current_owner() {
    let mut state = CursorLeaseState::default();
    let ruler = acquire(&mut state, CursorOwner::Ruler, CursorIcon::Crosshair);
    state.suspend(ruler);
    let screenshot = acquire(
      &mut state,
      CursorOwner::QuickScreenshot,
      CursorIcon::Crosshair,
    );
    assert_eq!(
      state.release(ruler),
      CursorTransition {
        state_changed: true,
        presentation: None,
      }
    );
    assert_eq!(state.active().unwrap().id, screenshot);
  }

  #[test]
  fn an_active_owner_must_be_explicitly_suspended_or_released() {
    let mut state = CursorLeaseState::default();
    let ruler = acquire(&mut state, CursorOwner::Ruler, CursorIcon::Crosshair);
    assert_eq!(
      state.acquire(CursorOwner::QuickScreenshot, CursorIcon::Crosshair),
      Err(CursorLeaseError::AlreadyOwned(ruler))
    );
    assert_eq!(state.resume(ruler), CursorTransition::ignored());
  }

  #[test]
  fn restoration_is_lifo_and_cannot_skip_a_newer_suspended_owner() {
    let mut state = CursorLeaseState::default();
    let ruler = acquire(&mut state, CursorOwner::Ruler, CursorIcon::Crosshair);
    let (screenshot, _) = state
      .transfer(ruler, CursorOwner::QuickScreenshot, CursorIcon::Crosshair)
      .unwrap();
    let (ocr, _) = state
      .transfer(screenshot, CursorOwner::TextRecognition, CursorIcon::IBeam)
      .unwrap();
    assert_eq!(state.restore(ocr, ruler), CursorTransition::ignored());
    assert_eq!(state.active().unwrap().id, ocr);
    assert!(state.restore(ocr, screenshot).state_changed);
    assert!(state.restore(screenshot, ruler).state_changed);
    assert_eq!(state.active().unwrap().id, ruler);
  }
}
