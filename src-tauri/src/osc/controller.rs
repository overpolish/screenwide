// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! State machine for screenshot-style region selection. This deliberately has
//! no rendering or platform dependencies; a host can present its draft while
//! only `Finished` commits a semantic capture rectangle.
use super::geometry::{Handle, Monitor, Point, Rect};
use super::gesture::{hit_test, Gesture, GestureKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControllerEvent {
  Changed {
    draft: Option<Rect>,
    kind: GestureKind,
  },
  Finished {
    committed: Option<Rect>,
    kind: GestureKind,
  },
  Cancelled {
    committed: Option<Rect>,
  },
}

#[derive(Debug)]
pub struct RegionController {
  monitor: Monitor,
  committed: Option<Rect>,
  draft: Option<Rect>,
  gesture: Option<Gesture>,
  aspect: Option<f64>,
}

impl RegionController {
  pub fn new(monitor: Monitor, committed: Option<Rect>, aspect: Option<f64>) -> Self {
    let committed = committed
      .map(|rect| rect.clamp(monitor).snap())
      .filter(|rect| rect.committed());
    Self {
      monitor,
      draft: committed,
      committed,
      gesture: None,
      aspect: clean_aspect(aspect),
    }
  }
  pub fn committed(&self) -> Option<Rect> {
    self.committed
  }
  #[cfg(test)]
  pub fn draft(&self) -> Option<Rect> {
    self.draft
  }
  pub fn set_aspect(&mut self, aspect: Option<f64>) {
    self.aspect = clean_aspect(aspect);
  }
  pub fn set_monitor(&mut self, monitor: Monitor) -> bool {
    if self.gesture.is_some() {
      return false;
    }
    self.monitor = monitor;
    self.committed = self
      .committed
      .map(|rect| rect.clamp(monitor).snap())
      .filter(|rect| rect.committed());
    self.draft = self.committed;
    true
  }
  pub fn set_committed(&mut self, rect: Option<Rect>) -> bool {
    if self.gesture.is_some() {
      return false;
    }
    self.committed = rect
      .map(|value| value.clamp(self.monitor).snap())
      .filter(|value| value.committed());
    self.draft = self.committed;
    true
  }
  #[cfg(test)]
  pub fn gesture_active(&self) -> bool {
    self.gesture.is_some()
  }
  pub fn hover_kind(&self, point: Point) -> GestureKind {
    self.kind_at(point)
  }
  fn kind_at(&self, point: Point) -> GestureKind {
    self
      .committed
      .and_then(|rect| hit_test(rect, point, 8.0))
      .map_or(GestureKind::Drawing, |handle| match handle {
        Handle::Body => GestureKind::Moving,
        handle => GestureKind::Resizing(handle),
      })
  }
  pub fn pointer_down(&mut self, point: Point) -> GestureKind {
    let kind = self.kind_at(point);
    let start_rect = self.committed.unwrap_or(Rect {
      origin: point,
      size: super::geometry::Size::default(),
    });
    self.gesture = Some(Gesture::begin(kind, point, start_rect, self.aspect));
    if kind == GestureKind::Drawing {
      self.draft = None;
    }
    kind
  }
  pub fn pointer_move(&mut self, point: Point, shift: bool) -> Option<ControllerEvent> {
    let gesture = self.gesture.as_mut()?;
    let kind = gesture.kind();
    let draft = gesture.update(point, self.monitor, shift);
    self.draft = Some(draft).filter(|rect| rect.committed());
    Some(ControllerEvent::Changed {
      draft: self.draft,
      kind,
    })
  }
  pub fn pointer_up(&mut self, point: Point, shift: bool) -> Option<ControllerEvent> {
    let gesture = self.gesture.take()?;
    let kind = gesture.kind();
    let draft = Some(gesture.finish(point, self.monitor, shift)).filter(|rect| rect.committed());
    self.committed = draft;
    self.draft = draft;
    Some(ControllerEvent::Finished {
      committed: draft,
      kind,
    })
  }
  pub fn cancel(&mut self) -> Option<ControllerEvent> {
    self.gesture.take()?;
    self.draft = self.committed;
    Some(ControllerEvent::Cancelled {
      committed: self.committed,
    })
  }
}
fn clean_aspect(aspect: Option<f64>) -> Option<f64> {
  aspect.filter(|value| value.is_finite() && *value > 0.0)
}

#[cfg(test)]
mod tests {
  use super::*;
  fn monitor() -> Monitor {
    Monitor {
      size: super::super::geometry::Size {
        width: 100.,
        height: 80.,
      },
    }
  }
  fn region() -> Rect {
    Rect {
      origin: Point { x: 20., y: 20. },
      size: super::super::geometry::Size {
        width: 30.,
        height: 20.,
      },
    }
  }
  #[test]
  fn draw_and_finish_semantics() {
    let mut c = RegionController::new(monitor(), None, None);
    c.pointer_down(Point { x: 10., y: 10. });
    assert!(c.pointer_move(Point { x: 30., y: 35. }, false).is_some());
    assert!(c.committed().is_none());
    let event = c.pointer_up(Point { x: 30., y: 35. }, false).unwrap();
    assert!(matches!(
      event,
      ControllerEvent::Finished {
        kind: GestureKind::Drawing,
        committed: Some(_)
      }
    ));
    assert!(c.committed().unwrap().committed());
  }
  #[test]
  fn move_and_resize_route_by_hit_test() {
    let mut c = RegionController::new(monitor(), Some(region()), None);
    c.pointer_down(Point { x: 35., y: 30. });
    assert!(matches!(
      c.pointer_up(Point { x: 45., y: 35. }, false),
      Some(ControllerEvent::Finished {
        kind: GestureKind::Moving,
        ..
      })
    ));
    c.pointer_down(Point { x: 65., y: 35. });
    assert!(matches!(
      c.pointer_up(Point { x: 90., y: 40. }, false),
      Some(ControllerEvent::Finished {
        kind: GestureKind::Resizing(Handle::East),
        ..
      })
    ));
  }
  #[test]
  fn outside_starts_redraw_and_small_is_empty() {
    let mut c = RegionController::new(monitor(), Some(region()), None);
    c.pointer_down(Point { x: 2., y: 2. });
    c.pointer_move(Point { x: 2.5, y: 2.5 }, false);
    assert!(c.draft().is_none());
    c.pointer_up(Point { x: 2.5, y: 2.5 }, false);
    assert!(c.committed().is_none());
  }
  #[test]
  fn cancel_restores_committed_and_shift_latches_resize() {
    let mut c = RegionController::new(monitor(), Some(region()), Some(1.));
    c.pointer_down(Point { x: 50., y: 40. });
    c.pointer_move(Point { x: 60., y: 50. }, true);
    c.pointer_move(Point { x: 70., y: 60. }, false);
    assert!(c.cancel().is_some());
    assert_eq!(c.committed(), Some(region()));
  }
  #[test]
  fn aspect_locked_drawing() {
    let mut c = RegionController::new(monitor(), None, Some(2.));
    c.pointer_down(Point { x: 10., y: 10. });
    c.pointer_move(Point { x: 30., y: 30. }, false);
    let draft = c.draft().unwrap();
    assert_eq!(draft.size.width, 40.);
    assert_eq!(draft.size.height, 20.);
  }
  #[test]
  fn shift_temporarily_frees_drawing_aspect() {
    let mut c = RegionController::new(monitor(), None, Some(2.));
    c.pointer_down(Point { x: 10., y: 10. });
    c.pointer_move(Point { x: 30., y: 30. }, false);
    assert_eq!(c.draft().unwrap().size.width, 40.);
    c.pointer_move(Point { x: 30., y: 30. }, true);
    assert_eq!(c.draft().unwrap().size.width, 20.);
    c.pointer_move(Point { x: 30., y: 30. }, false);
    assert_eq!(c.draft().unwrap().size.width, 40.);
  }
  #[test]
  fn external_sync_is_guarded_and_empty_resets() {
    let mut c = RegionController::new(monitor(), Some(region()), None);
    let incoming = Rect {
      origin: Point { x: 90., y: 70. },
      size: super::super::geometry::Size {
        width: 30.,
        height: 30.,
      },
    };
    assert!(c.set_committed(Some(incoming)));
    assert!(c.committed().unwrap().right() <= 100.);
    c.pointer_down(Point { x: 95., y: 75. });
    assert!(!c.set_committed(None));
    assert!(c.committed().is_some());
    assert!(c.gesture_active());
    c.cancel();
    assert!(c.set_committed(None));
    assert!(c.committed().is_none());
    assert!(c.draft().is_none());
  }
  #[test]
  fn monitor_changes_reclamp_committed_state_and_wait_for_active_gestures() {
    let mut c = RegionController::new(monitor(), Some(region()), None);
    c.pointer_down(Point { x: 35., y: 30. });
    assert!(!c.set_monitor(Monitor {
      size: super::super::geometry::Size {
        width: 40.,
        height: 30.,
      },
    }));
    c.cancel();
    assert!(c.set_monitor(Monitor {
      size: super::super::geometry::Size {
        width: 40.,
        height: 30.,
      },
    }));
    let committed = c.committed().unwrap();
    assert!(committed.right() <= 40.);
    assert!(committed.bottom() <= 30.);
  }
  #[test]
  fn hover_reports_priority_without_mutation() {
    let mut c = RegionController::new(monitor(), Some(region()), None);
    let before = (c.committed(), c.draft(), c.gesture_active());
    assert_eq!(
      c.hover_kind(Point { x: 20., y: 20. }),
      GestureKind::Resizing(Handle::NorthWest)
    );
    assert_eq!(c.hover_kind(Point { x: 35., y: 30. }), GestureKind::Moving);
    assert_eq!(c.hover_kind(Point { x: 2., y: 2. }), GestureKind::Drawing);
    assert_eq!((c.committed(), c.draft(), c.gesture_active()), before);
    assert_eq!(
      c.pointer_down(Point { x: 35., y: 30. }),
      GestureKind::Moving
    );
  }
}
