// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use rayon::prelude::*;

impl RulerState {
  /// Detects element boxes for every tolerance once Ruler is already on
  /// screen. Snapping, the radius tool and centre lines see no boxes until
  /// this lands, which normally happens before the user can finish a drag.
  /// Detection runs outside the session lock so hover keeps responding.
  pub(in crate::ruler) fn analyze_boxes(&self, generation: u64) -> bool {
    let inputs = {
      let session = self
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      if !session.active || session.generation != generation {
        return false;
      }
      session
        .displays
        .iter()
        .map(|snapshot| (snapshot.display.id, Arc::clone(&snapshot.gradients)))
        .collect::<Vec<_>>()
    };
    let detected = inputs
      .into_par_iter()
      .map(|(id, maps)| {
        let (clear, (balanced, subtle)) = rayon::join(
          || detect_boxes(&maps, Tolerance::ClearEdges.threshold()),
          || {
            rayon::join(
              || detect_boxes(&maps, Tolerance::Balanced.threshold()),
              || detect_boxes(&maps, Tolerance::SubtleEdges.threshold()),
            )
          },
        );
        (id, [clear, balanced, subtle])
      })
      .collect::<Vec<_>>();

    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active || session.generation != generation {
      return false;
    }
    for (id, boxes) in detected {
      if let Some(snapshot) = session
        .displays
        .iter_mut()
        .find(|snapshot| snapshot.display.id == id)
      {
        snapshot.boxes_by_tolerance = boxes;
      }
    }
    let tolerance = session.tolerance;
    session.boxes = session
      .displays
      .iter()
      .flat_map(|snapshot| detected_boxes(snapshot, tolerance))
      .collect();
    session.center_aid_cache = None;
    true
  }
}
