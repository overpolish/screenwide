// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn radii(&self) -> Vec<RulerRadiusVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let mut radii = session
      .document
      .radii
      .iter()
      .map(|radius| RulerRadiusVisual {
        id: radius.id,
        display_id: radius.display_id,
        bounds: radius.bounds,
        corner: radius.corner,
        radius: radius.radius,
        low_confidence: radius.low_confidence,
        draft: false,
        hovered: session.hovered_target == Some(HoverTarget::Radius(radius.id)),
        hover_alpha: hover_alpha(&session, HoverTarget::Radius(radius.id), now),
        label_anchor: radius.label.anchor,
        label_hidden: radius.label.hidden,
      })
      .collect::<Vec<_>>();
    if let Some(visual) = session.radius.and_then(|gesture| gesture.visual) {
      let already_stamped = radii.iter().any(|radius| {
        radius.display_id == visual.display_id
          && radius.bounds == visual.bounds
          && radius.corner == visual.corner
          && (radius.radius - visual.radius).abs() < f64::EPSILON
      });
      if !already_stamped {
        radii.push(visual);
      }
    }
    radii
  }

  pub(crate) fn center_aids(&self) -> (Vec<RulerCenterlineVisual>, Vec<RulerInnerObjectVisual>) {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.centerlines_visible {
      return (Vec::new(), Vec::new());
    }
    center_aid_visuals(&mut session, Instant::now())
  }

  pub(crate) fn begin_radius(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.radius.is_some() || session.range.is_some() || session.guide.is_some() {
      return None;
    }
    let visual = session.visual?;
    session.drag = Drag::default();
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    session.radius = Some(RadiusGesture { visual: None });
    update_radius(&mut session, pointer_from_visual(visual));
    Some(visual)
  }

  pub(crate) fn cancel_radius(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.radius.take()?;
    session.visual
  }

  pub(crate) fn begin_guide(&self, axis: GuideAxis) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.guide.is_some() || session.range.is_some() || session.radius.is_some() {
      return None;
    }
    let visual = session.visual?;
    let pointer = pointer_from_visual(visual);
    session.drag = Drag::default();
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    session.guide = Some(GuideGesture {
      visual: RulerGuideVisual {
        id: 0,
        display_id: pointer.display_id,
        axis,
        position: guide_pointer_position(axis, pointer.world),
        draft: true,
        hovered: false,
        hover_alpha: 0.0,
      },
      snapped: false,
    });
    update_guide(&mut session, pointer);
    Some(visual)
  }

  pub(crate) fn cancel_guide(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.guide.take()?;
    session.visual
  }

  pub(crate) fn begin_range(&self, axis: RangeAxis) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.range.is_some() || session.guide.is_some() || session.radius.is_some() {
      return None;
    }
    let visual = session.visual?;
    let pointer = pointer_from_visual(visual);
    let axis = match axis {
      RangeAxis::Horizontal => ProbeAxis::Horizontal,
      RangeAxis::Vertical => ProbeAxis::Vertical,
    };
    let start_probe = automatic_probes(&session, pointer)?
      .into_iter()
      .find(|probe| probe.axis == axis)?;
    session.hovered_target = None;
    session.label_drag = None;
    session.range = Some(RangeGesture {
      axis,
      start_pointer: pointer,
      start_probe,
      draft: RulerProbeVisual {
        draft: true,
        ..start_probe
      },
    });
    Some(visual)
  }
}
