// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(crate) fn redo(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let next = session.redo.pop()?;
    let current = std::mem::replace(&mut session.document, next);
    session.undo.push(current);
    trim_history(&mut session.undo);
    clear_transient_artifact_state(&mut session);
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn measurements(&self) -> Vec<RulerMeasurementVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    measurement_visuals(&mut session, Instant::now())
  }

  pub(crate) fn viewports(&self) -> Vec<RulerViewportVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .displays
      .iter()
      .map(|snapshot| RulerViewportVisual {
        display_id: snapshot.display.id,
        viewport: snapshot.viewport,
      })
      .collect()
  }

  pub(crate) fn probes(&self) -> Vec<RulerProbeVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    probe_visuals(&session)
  }

  pub(crate) fn guides(&self) -> Vec<RulerGuideVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let mut guides = session
      .document
      .guides
      .iter()
      .map(|guide| RulerGuideVisual {
        id: guide.id,
        display_id: guide.display_id,
        axis: guide.axis,
        position: guide.position,
        draft: false,
        hovered: session.hovered_target == Some(HoverTarget::Guide(guide.id)),
        hover_alpha: hover_alpha(&session, HoverTarget::Guide(guide.id), now),
      })
      .collect::<Vec<_>>();
    if let Some(gesture) = session.guide {
      guides.push(gesture.visual);
    }
    guides
  }

  pub(crate) fn guide_gaps(&self) -> Vec<RulerGuideGapVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    guide_gap_visuals(&session)
  }

  pub(crate) fn hovered_guide_axis(&self) -> Option<GuideAxis> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let HoverTarget::Guide(id) = session.hovered_target? else {
      return None;
    };
    session
      .document
      .guides
      .iter()
      .find(|guide| guide.id == id)
      .map(|guide| guide.axis)
  }
}
