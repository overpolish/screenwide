// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(in crate::ruler) fn begin(&self) -> u64 {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    session.active = true;
    session.generation
  }

  pub(in crate::ruler) fn is_current(&self, generation: u64) -> bool {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.active && session.generation == generation
  }

  pub(in crate::ruler) fn install(
    &self,
    generation: u64,
    displays: &[DesktopDisplay],
    snapshots: &[(u32, CapturedImage)],
  ) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active || session.generation != generation {
      return false;
    }
    session.displays = displays
      .iter()
      .filter_map(|display| {
        snapshots
          .iter()
          .find(|(id, _)| *id == display.id)
          .map(|(_, image)| {
            let gradients = compute_gradients(&image.rgba, image.width, image.height);
            let probes = ProbeIndex::new(&gradients, Tolerance::Balanced.threshold());
            let boxes_by_tolerance = [
              detect_boxes(&gradients, Tolerance::ClearEdges.threshold()),
              detect_boxes(&gradients, Tolerance::Balanced.threshold()),
              detect_boxes(&gradients, Tolerance::SubtleEdges.threshold()),
            ];
            DisplaySnapshot {
              display: *display,
              image: image.clone(),
              gradients,
              probes,
              boxes_by_tolerance,
              viewport: Viewport::default(),
            }
          })
      })
      .collect();
    session.tolerance = Tolerance::Balanced;
    session.boxes = session
      .displays
      .iter()
      .flat_map(|snapshot| detected_boxes(snapshot, Tolerance::Balanced))
      .collect();
    session.visual = None;
    session.copied_until = None;
    session.tolerance_until = None;
    session.option_active = false;
    session.drag = Drag::default();
    session.document = Document::default();
    session.undo.clear();
    session.redo.clear();
    session.hovered_target = None;
    session.hover_exit = None;
    session.label_drag = None;
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.centerlines_visible = true;
    session.center_aid_cache = None;
    session.settle = None;
    session.displays.len() == displays.len()
  }

  pub(crate) fn map_pointer(&self, screen: Point) -> Option<RulerPointer> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    map_pointer(&session.displays, screen)
  }

  pub(crate) fn hover(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    let rgba = sample(&session.displays, pointer.world)?;
    let now = Instant::now();
    let copied = session.copied_until.is_some_and(|deadline| deadline > now);
    let crosshair = session.visual.is_some_and(|visual| visual.crosshair);
    let hovered_target = if session.range.is_none()
      && session.guide.is_none()
      && session.guide_drag.is_none()
      && session.radius.is_none()
    {
      hit_test_artifact(&session, pointer)
    } else {
      None
    };
    update_hover_target(&mut session, hovered_target, Instant::now());
    update_range(&mut session, pointer);
    update_guide(&mut session, pointer);
    update_radius(&mut session, pointer);
    let visual = RulerVisual {
      point: pointer.world,
      screen_point: pointer.screen,
      display_id: pointer.display_id,
      zoom: pointer.zoom,
      rgba,
      crosshair,
      copied,
    };
    session.visual = Some(visual);
    Some(visual)
  }

  pub(crate) fn pointer_down(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active || sample(&session.displays, pointer.world).is_none() {
      return None;
    }
    if session.range.is_some() {
      update_range(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      let guide = session.guide?.visual;
      record_history(&mut session);
      session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
      let id = session.document.next_id;
      session.document.guides.push(GuideArtifact {
        id,
        display_id: guide.display_id,
        axis: guide.axis,
        position: guide.position,
        anchor: guide_cross_axis_position(guide.axis, pointer.world),
      });
      reconcile_guide_gaps(&mut session.document);
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      if let Some(radius) = session.radius.and_then(|gesture| gesture.visual) {
        let duplicate = session.document.radii.iter().any(|item| {
          item.display_id == radius.display_id
            && item.bounds == radius.bounds
            && item.corner == radius.corner
            && (item.radius - radius.radius).abs() < f64::EPSILON
        });
        if !duplicate {
          record_history(&mut session);
          session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
          let id = session.document.next_id;
          session.document.radii.push(RadiusArtifact {
            id,
            display_id: radius.display_id,
            bounds: radius.bounds,
            corner: radius.corner,
            radius: radius.radius,
            low_confidence: radius.low_confidence,
            label: ArtifactLabel::default(),
          });
        }
      }
      return refresh_visual(&mut session, pointer);
    }
    if let Some(HoverTarget::Guide(id)) = session.hovered_target {
      let original = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == id)
        .copied()?;
      session.drag = Drag::default();
      session.label_drag = None;
      session.settle = None;
      session.guide_drag = Some(GuideDrag {
        id,
        start_screen: pointer.screen,
        original,
        changed: false,
        snapped: false,
      });
      return refresh_visual(&mut session, pointer);
    }
    session.drag = Drag {
      pending: Some(pointer),
      ..Default::default()
    };
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn pointer_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    if session.range.is_some() {
      update_range(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.guide_drag.is_some() {
      update_guide_drag_session(&mut session, pointer)?;
      return refresh_visual(&mut session, pointer);
    }
    if session.drag.start.is_none() {
      let pending = session.drag.pending?;
      if (pointer.screen.x - pending.screen.x).hypot(pointer.screen.y - pending.screen.y)
        >= DRAG_THRESHOLD
      {
        session.drag.start = Some(pending);
        session.hovered_target = None;
      }
    }
    if let Some(start) = session.drag.start {
      session.drag.draft = Some(ordered_rect(start.world, pointer.world));
    }
    refresh_visual(&mut session, pointer)
  }
}
