// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn refresh_visual(
  session: &mut Session,
  pointer: RulerPointer,
) -> Option<RulerVisual> {
  let rgba = sample(&session.displays, pointer.world)
    .or_else(|| session.visual.map(|visual| visual.rgba))?;
  if session.drag.start.is_none()
    && session.range.is_none()
    && session.guide.is_none()
    && session.guide_drag.is_none()
    && session.radius.is_none()
    && session.label_drag.is_none()
  {
    let hovered_target = hit_test_artifact(session, pointer);
    update_hover_target(session, hovered_target, Instant::now());
  }
  update_range(session, pointer);
  update_guide(session, pointer);
  update_radius(session, pointer);
  let now = Instant::now();
  let visual = RulerVisual {
    point: pointer.world,
    screen_point: pointer.screen,
    display_id: pointer.display_id,
    zoom: pointer.zoom,
    rgba,
    crosshair: session.visual.is_some_and(|visual| visual.crosshair),
    copied: session.copied_until.is_some_and(|deadline| deadline > now),
  };
  session.visual = Some(visual);
  Some(visual)
}

pub(in crate::ruler::snapshot) fn update_hover_target(
  session: &mut Session,
  target: Option<HoverTarget>,
  now: Instant,
) {
  if session.hovered_target == target {
    expire_hover_exit(session, now);
    return;
  }
  session.hover_exit = match (session.hovered_target, target) {
    (Some(previous), None) => Some(HoverExit {
      target: previous,
      started: now,
    }),
    _ => None,
  };
  session.hovered_target = target;
}

pub(in crate::ruler::snapshot) fn expire_hover_exit(session: &mut Session, now: Instant) {
  if session
    .hover_exit
    .is_some_and(|exit| now.saturating_duration_since(exit.started) >= HOVER_EXIT_DURATION)
  {
    session.hover_exit = None;
  }
}

pub(in crate::ruler::snapshot) fn hover_alpha(
  session: &Session,
  target: HoverTarget,
  now: Instant,
) -> f32 {
  if session.hovered_target == Some(target) {
    return 1.0;
  }
  let Some(exit) = session.hover_exit.filter(|exit| exit.target == target) else {
    return 0.0;
  };
  let progress =
    now.saturating_duration_since(exit.started).as_secs_f32() / HOVER_EXIT_DURATION.as_secs_f32();
  (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

pub(in crate::ruler::snapshot) fn pointer_from_visual(visual: RulerVisual) -> RulerPointer {
  RulerPointer {
    world: visual.point,
    screen: visual.screen_point,
    display_id: visual.display_id,
    zoom: visual.zoom,
  }
}

pub(in crate::ruler::snapshot) fn map_pointer(
  displays: &[DisplaySnapshot],
  screen: Point,
) -> Option<RulerPointer> {
  let snapshot = displays.iter().find(|snapshot| {
    let display = snapshot.display;
    screen.x >= display.origin.x
      && screen.y >= display.origin.y
      && screen.x < display.origin.x + display.size.width
      && screen.y < display.origin.y + display.size.height
  })?;
  let local_screen = Point {
    x: screen.x - snapshot.display.origin.x,
    y: screen.y - snapshot.display.origin.y,
  };
  let local_world = snapshot.viewport.screen_to_source(local_screen);
  Some(RulerPointer {
    world: Point {
      x: snapshot.display.origin.x + local_world.x,
      y: snapshot.display.origin.y + local_world.y,
    },
    screen,
    display_id: snapshot.display.id,
    zoom: snapshot.viewport.zoom,
  })
}

pub(in crate::ruler::snapshot) fn measurement_visuals(
  session: &mut Session,
  now: Instant,
) -> Vec<RulerMeasurementVisual> {
  let mut visuals = session
    .document
    .measurements
    .iter()
    .map(|measurement| RulerMeasurementVisual {
      id: measurement.id,
      bounds: measurement.bounds,
      draft: false,
      animating: false,
      hovered: session.hovered_target == Some(HoverTarget::Measurement(measurement.id)),
      hover_alpha: hover_alpha(session, HoverTarget::Measurement(measurement.id), now),
      label_anchor: measurement.label.anchor,
      label_hidden: measurement.label.hidden,
    })
    .collect::<Vec<_>>();
  if let Some(draft) = session.drag.draft {
    visuals.push(RulerMeasurementVisual {
      id: 0,
      bounds: draft,
      draft: true,
      animating: false,
      hovered: false,
      hover_alpha: 0.0,
      label_anchor: None,
      label_hidden: false,
    });
    return visuals;
  }
  let Some(settle) = session.settle else {
    return visuals;
  };
  let progress = now.duration_since(settle.started).as_secs_f64() / SETTLE_DURATION.as_secs_f64();
  if progress >= 1.0 {
    session.settle = None;
    return visuals;
  }
  let rest = progress - 1.0;
  let eased = 1.0 + (SETTLE_OVERSHOOT + 1.0) * rest.powi(3) + SETTLE_OVERSHOOT * rest.powi(2);
  if let Some(visual) = visuals.iter_mut().find(|item| item.id == settle.id) {
    visual.bounds = mix_rect(settle.from, settle.to, eased);
    visual.animating = true;
  }
  visuals
}

pub(in crate::ruler::snapshot) fn measurement_device_scale(
  displays: &[DisplaySnapshot],
  bounds: Rect,
) -> f64 {
  let center = Point {
    x: bounds.origin.x + bounds.size.width * 0.5,
    y: bounds.origin.y + bounds.size.height * 0.5,
  };
  displays
    .iter()
    .find(|snapshot| {
      center.x >= snapshot.display.origin.x
        && center.y >= snapshot.display.origin.y
        && center.x < snapshot.display.origin.x + snapshot.display.size.width
        && center.y < snapshot.display.origin.y + snapshot.display.size.height
    })
    .map_or(1.0, |snapshot| {
      f64::from(snapshot.image.width.max(1)) / snapshot.display.size.width.max(1.0)
    })
}

pub(in crate::ruler::snapshot) fn center_aid_visuals(
  session: &mut Session,
  now: Instant,
) -> (Vec<RulerCenterlineVisual>, Vec<RulerInnerObjectVisual>) {
  if session.settle.is_none() {
    if let Some(cache) = &session.center_aid_cache {
      if cache.document == session.document {
        return (cache.lines.clone(), cache.objects.clone());
      }
    }
  }
  let measurements = session.document.measurements.clone();
  let drawn = measurement_visuals(session, now);
  let mut lines = Vec::with_capacity(measurements.len());
  let mut objects = Vec::new();
  for measurement in &measurements {
    let Some(visual) = drawn.iter().find(|item| item.id == measurement.id) else {
      continue;
    };
    let peers = measurements
      .iter()
      .filter(|peer| peer.id != measurement.id)
      .map(|peer| peer.bounds)
      .collect::<Vec<_>>();
    let analysis = centerlines::analyze(
      measurement.bounds,
      &session.boxes,
      &peers,
      measurement_device_scale(&session.displays, measurement.bounds),
    );
    lines.push(RulerCenterlineVisual {
      id: measurement.id,
      bounds: visual.bounds,
      x_accent: analysis.x_accent,
      y_accent: analysis.y_accent,
    });
    if !visual.animating {
      objects.extend(
        analysis
          .objects
          .into_iter()
          .map(|object| RulerInnerObjectVisual {
            owner_id: measurement.id,
            bounds: object.bounds,
            aligned_x: object.aligned_x,
            aligned_y: object.aligned_y,
          }),
      );
    }
  }
  if session.settle.is_none() {
    session.center_aid_cache = Some(CenterAidCache {
      document: session.document.clone(),
      lines: lines.clone(),
      objects: objects.clone(),
    });
  }
  (lines, objects)
}
