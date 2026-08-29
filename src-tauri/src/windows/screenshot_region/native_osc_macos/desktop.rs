// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use crate::osc::{
  controller::ControllerEvent,
  desktop::{global_region, local_projection, DesktopDisplay},
  geometry::{Monitor, Point, Rect, Size},
};

use super::{ffi, Context};

const MAX_DISPLAYS: usize = 16;

#[derive(Clone, Debug)]
pub struct DesktopBinding {
  pub displays: Vec<DesktopDisplay>,
  pub anchor_id: u32,
  pub size: Size,
  pub layout_changed: bool,
}

impl DesktopBinding {
  pub(super) fn anchor(&self) -> Option<DesktopDisplay> {
    self
      .displays
      .iter()
      .copied()
      .find(|display| display.id == self.anchor_id)
  }

  pub(super) fn virtual_monitor(&self) -> Monitor {
    Monitor { size: self.size }
  }

  pub fn project_local(&self, local: Rect) -> Option<Rect> {
    self.anchor().map(|anchor| global_region(anchor, local))
  }

  pub fn reconcile_local(&self, local: Rect) -> Option<DesktopRegion> {
    let global = self.project_local(local)?;
    let (global, owner) =
      crate::osc::desktop::reconcile_region(&self.displays, Some(self.anchor_id), global)?;
    Some(DesktopRegion {
      anchor_local: local_projection(self.anchor()?, global),
      owner_local: local_projection(owner, global),
      global,
      owner_id: owner.id,
    })
  }
}

#[derive(Clone, Copy, Debug)]
pub struct DesktopRegion {
  pub anchor_local: Rect,
  pub owner_local: Rect,
  pub global: Rect,
  pub owner_id: u32,
}

pub fn configure_window(view: *mut c_void, anchor_id: u32) -> Result<DesktopBinding, String> {
  let mut native = [ffi::NativeDesktopDisplay::default(); MAX_DISPLAYS];
  let mut width = 0.0;
  let mut height = 0.0;
  let mut resolved_anchor_id = anchor_id;
  let mut layout_changed = 0;
  let count = unsafe {
    ffi::screenwide_region_osc_configure_desktop(
      view,
      anchor_id,
      native.as_mut_ptr(),
      native.len(),
      &mut width,
      &mut height,
      &mut resolved_anchor_id,
      &mut layout_changed,
    )
  };
  let displays = native[..count.min(native.len())]
    .iter()
    .map(|display| DesktopDisplay {
      id: display.id,
      origin: Point {
        x: display.x,
        y: display.y,
      },
      size: Size {
        width: display.width,
        height: display.height,
      },
      scale: display.scale,
    })
    .collect::<Vec<_>>();
  if !displays
    .iter()
    .any(|display| display.id == resolved_anchor_id)
  {
    return Err(format!(
      "AppKit could not resolve a Region monitor after losing: {anchor_id}"
    ));
  }
  let size = Size { width, height };
  if displays.is_empty() || !size.valid() || width <= 0.0 || height <= 0.0 {
    return Err("AppKit returned no valid desktop displays".to_owned());
  }
  Ok(DesktopBinding {
    displays,
    anchor_id: resolved_anchor_id,
    size,
    layout_changed: layout_changed != 0,
  })
}

impl Context {
  // The native host covers the AppKit screen union, so its top-left input is
  // already in controller coordinates.
  pub(super) fn controller_point(&self, point: Point) -> Point {
    point
  }

  pub(super) fn project_event(&self, event: ControllerEvent) -> (ControllerEvent, Option<u32>) {
    let Ok(mut desktop) = self.desktop.lock() else {
      return (event, None);
    };
    let Some(binding) = desktop.as_mut() else {
      return (event, None);
    };
    let owner_for = |region: Rect| {
      crate::osc::desktop::owner_for_region(&binding.displays, Some(binding.anchor_id), region)
        .or_else(|| binding.anchor())
    };
    let owner = match event {
      ControllerEvent::Changed { draft, .. } => draft.and_then(owner_for),
      ControllerEvent::Finished { committed, .. } | ControllerEvent::Cancelled { committed } => {
        committed.and_then(owner_for)
      }
    };
    let projected = match event {
      ControllerEvent::Changed { draft, kind } => ControllerEvent::Changed {
        draft: draft
          .zip(owner)
          .map(|(region, owner)| local_projection(owner, region)),
        kind,
      },
      ControllerEvent::Finished { committed, kind } => ControllerEvent::Finished {
        committed: committed
          .zip(owner)
          .map(|(region, owner)| local_projection(owner, region)),
        kind,
      },
      ControllerEvent::Cancelled { committed } => ControllerEvent::Cancelled {
        committed: committed
          .zip(owner)
          .map(|(region, owner)| local_projection(owner, region)),
      },
    };
    if let Some(owner) = owner {
      binding.anchor_id = owner.id;
    }
    (projected, Some(binding.anchor_id))
  }

  // OSC drawing stays in desktop-union coordinates. Only the semantic event
  // crossing into persisted frontend state is projected back to the anchor.
  pub(super) fn project_result(&self, _result: &mut super::NativeOscResult) {}
}

pub(super) fn global_committed(binding: &DesktopBinding, local: Option<Rect>) -> Option<Rect> {
  local.and_then(|region| binding.project_local(region))
}
