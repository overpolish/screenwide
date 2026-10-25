// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a press on the island asks of the Ruler: each is sent to the overlay
//! as the key that does the same would be, so the island and the keys are
//! one set of gestures.

use screenwide_osc::protocol::InputPhase;
use tauri::{AppHandle, Manager};

use super::super::{ruler_windows, RulerState};
use super::{shown, RulerIslandAction, RulerIslandState, RulerTolerance, RulerTool};
use crate::app_windows::{self, screenshot_region::region_osc as native_region};

/// Sends `phases` to the Ruler's overlay in order, as its keys would, on the
/// thread that owns it.
fn dispatch(app: &AppHandle, phases: Vec<InputPhase>) {
  let Some(window) = ruler_windows(app).into_iter().next() else {
    return;
  };
  let result = app.run_on_main_thread(move || {
    for phase in phases {
      native_region::ruler_phase(&window, phase as u32);
    }
  });
  if let Err(error) = result {
    eprintln!("Could not reach the Ruler from the island: {error}");
  }
}

#[tauri::command]
pub fn ruler_island_state() -> Option<RulerIslandState> {
  *shown()
}

/// Puts the tool in hand down and picks up `tool`. A gesture can only start
/// once the pointer has been over the desktop, so a pick that cannot start
/// leaves the island showing the tool that is still in hand.
#[tauri::command]
pub fn set_ruler_tool(app: AppHandle, tool: RulerTool) {
  let Some(current) = app.state::<RulerState>().island_state() else {
    return;
  };
  if current.tool == tool {
    return;
  }
  let mut phases = Vec::with_capacity(2);
  match current.tool {
    RulerTool::Measure => {}
    RulerTool::VerticalGuide | RulerTool::HorizontalGuide => {
      phases.push(InputPhase::RulerCancelGuide);
    }
    RulerTool::Radius => phases.push(InputPhase::RulerCancelRadius),
  }
  match tool {
    RulerTool::Measure => {}
    RulerTool::VerticalGuide => phases.push(InputPhase::RulerBeginVerticalGuide),
    RulerTool::HorizontalGuide => phases.push(InputPhase::RulerBeginHorizontalGuide),
    RulerTool::Radius => phases.push(InputPhase::RulerBeginRadius),
  }
  dispatch(&app, phases);
}

/// Steps the edge tolerance round to `tolerance`, as its key steps it, so
/// the overlay names the one it lands on.
#[tauri::command]
pub fn set_ruler_tolerance(app: AppHandle, tolerance: RulerTolerance) {
  let Some(current) = app.state::<RulerState>().island_state() else {
    return;
  };
  let order = [
    RulerTolerance::ClearEdges,
    RulerTolerance::Balanced,
    RulerTolerance::SubtleEdges,
  ];
  let index = |value| order.iter().position(|item| *item == value).unwrap_or(0);
  let steps = (index(tolerance) + order.len() - index(current.tolerance)) % order.len();
  dispatch(&app, vec![InputPhase::RulerCycleTolerance; steps]);
}

#[tauri::command]
pub fn ruler_island_action(app: AppHandle, action: RulerIslandAction) {
  dispatch(&app, vec![action.phase()]);
}

/// A press on the island is done with: the keyboard goes back to the
/// overlay, where the Ruler's keys work.
#[tauri::command]
pub fn return_ruler_keyboard(app: AppHandle) {
  if shown().is_none() {
    return;
  }
  let Some(window) = ruler_windows(&app).into_iter().next() else {
    return;
  };
  let island = app_windows::island::window(&app);
  let result = app.run_on_main_thread(move || {
    // macOS: the island holds the keyboard only if a view on it took it.
    #[cfg(target_os = "macos")]
    if !island
      .as_ref()
      .is_some_and(|island| app_windows::platform::is_key_panel(island).unwrap_or(false))
    {
      return;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = island;
    native_region::focus_ruler_input(&window);
  });
  if let Err(error) = result {
    eprintln!("Could not reach the main thread to move keyboard focus: {error}");
  }
}
