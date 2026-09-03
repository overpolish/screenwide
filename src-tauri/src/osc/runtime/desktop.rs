// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::{
  controller::ControllerEvent,
  desktop::{local_projection, DesktopBinding},
};

pub fn project_desktop_event(
  binding: &DesktopBinding,
  event: ControllerEvent,
) -> (ControllerEvent, Option<u32>) {
  let anchor = binding.anchor();
  let projected = match event {
    ControllerEvent::Changed { draft, kind } => ControllerEvent::Changed {
      draft: draft
        .zip(anchor)
        .map(|(region, anchor)| local_projection(anchor, region)),
      kind,
    },
    ControllerEvent::Finished { committed, kind } => ControllerEvent::Finished {
      committed: committed
        .zip(anchor)
        .map(|(region, anchor)| local_projection(anchor, region)),
      kind,
    },
    ControllerEvent::Cancelled { committed } => ControllerEvent::Cancelled {
      committed: committed
        .zip(anchor)
        .map(|(region, anchor)| local_projection(anchor, region)),
    },
  };
  (projected, anchor.map(|anchor| anchor.id))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::osc::{
    desktop::DesktopDisplay,
    geometry::{Point, Rect, Size},
    gesture::GestureKind,
  };

  #[test]
  fn projection_keeps_semantic_geometry_relative_to_the_session_anchor() {
    let binding = DesktopBinding {
      displays: vec![
        DesktopDisplay {
          id: 1,
          origin: Point { x: 0.0, y: 0.0 },
          size: Size {
            width: 100.0,
            height: 80.0,
          },
          scale: 2.0,
        },
        DesktopDisplay {
          id: 2,
          origin: Point { x: 100.0, y: 0.0 },
          size: Size {
            width: 120.0,
            height: 80.0,
          },
          scale: 1.0,
        },
      ],
      anchor_id: 1,
      size: Size {
        width: 220.0,
        height: 80.0,
      },
      layout_changed: false,
    };
    let event = ControllerEvent::Changed {
      draft: Some(Rect::from_xywh(90.0, 10.0, 80.0, 30.0)),
      kind: GestureKind::Moving,
    };

    let (projected, owner) = project_desktop_event(&binding, event);

    assert_eq!(owner, Some(1));
    assert_eq!(binding.anchor_id, 1);
    assert_eq!(
      projected,
      ControllerEvent::Changed {
        draft: Some(Rect::from_xywh(90.0, 10.0, 80.0, 30.0)),
        kind: GestureKind::Moving,
      }
    );
  }
}
