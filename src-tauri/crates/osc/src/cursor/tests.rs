// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
