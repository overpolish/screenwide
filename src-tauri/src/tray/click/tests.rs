// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::tray::TrayIconId;
use tauri::{PhysicalPosition, Rect};

use super::*;

fn click(button: MouseButton, button_state: MouseButtonState) -> TrayIconEvent {
  TrayIconEvent::Click {
    id: TrayIconId::new("tray"),
    position: PhysicalPosition::default(),
    rect: Rect::default(),
    button,
    button_state,
  }
}

fn double_click() -> TrayIconEvent {
  TrayIconEvent::DoubleClick {
    id: TrayIconId::new("tray"),
    position: PhysicalPosition::default(),
    rect: Rect::default(),
    button: MouseButton::Left,
  }
}

const PRESS: MouseButtonState = MouseButtonState::Down;
const RELEASE: MouseButtonState = MouseButtonState::Up;

#[test]
fn a_left_click_toggles_on_release() {
  let filter = ClickFilter::default();
  assert_eq!(filter.action(&click(MouseButton::Left, PRESS), false), None);
  assert_eq!(
    filter.action(&click(MouseButton::Left, RELEASE), false),
    Some(ClickAction::ToggleRecordingControls)
  );
}

#[test]
fn a_right_click_is_left_to_the_tray_menu() {
  let filter = ClickFilter::default();
  assert_eq!(
    filter.action(&click(MouseButton::Right, PRESS), false),
    None
  );
  assert_eq!(
    filter.action(&click(MouseButton::Right, RELEASE), false),
    None
  );
}

#[test]
fn a_double_click_toggles_once() {
  let filter = ClickFilter::default();
  let actions: Vec<_> = [
    click(MouseButton::Left, PRESS),
    click(MouseButton::Left, RELEASE),
    double_click(),
    click(MouseButton::Left, RELEASE),
  ]
  .iter()
  .filter_map(|event| filter.action(event, false))
  .collect();
  assert_eq!(actions, [ClickAction::ToggleRecordingControls]);
}

#[test]
fn a_control_click_opens_the_menu_without_toggling() {
  let filter = ClickFilter::default();
  assert_eq!(
    filter.action(&click(MouseButton::Left, PRESS), true),
    Some(ClickAction::ShowMenu)
  );
  assert_eq!(
    filter.action(&click(MouseButton::Left, RELEASE), true),
    None
  );
}

#[test]
fn a_release_the_menu_swallowed_does_not_eat_the_next_click() {
  let filter = ClickFilter::default();
  filter.action(&click(MouseButton::Left, PRESS), true);
  assert_eq!(filter.action(&click(MouseButton::Left, PRESS), false), None);
  assert_eq!(
    filter.action(&click(MouseButton::Left, RELEASE), false),
    Some(ClickAction::ToggleRecordingControls)
  );
}
