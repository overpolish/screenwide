// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Temporary reproduction trace. Remove after the wrapping fix is verified.
//! Stores geometry only, replacing the previous drag's trace each time.

use super::{contains, intersection_over, Rect, CONTAINMENT_SLACK, EDGE_SEARCH};
use serde_json::json;

fn edges(rect: Rect) -> [f64; 4] {
  [rect.origin.x, rect.origin.y, rect.right(), rect.bottom()]
}

pub(super) fn record(boxes: &[Rect], drag: Rect, selected: Rect, result: Rect, branch: &str) {
  let expanded = |slack: f64| {
    Rect::from_xywh(
      drag.origin.x - slack,
      drag.origin.y - slack,
      drag.size.width + slack * 2.0,
      drag.size.height + slack * 2.0,
    )
  };
  let containment = expanded(CONTAINMENT_SLACK);
  let search = expanded(EDGE_SEARCH);
  let candidates = boxes
    .iter()
    .enumerate()
    .filter_map(|(index, candidate)| {
      let included = contains(containment, *candidate);
      if !included && intersection_over(search, *candidate) == 0.0 {
        return None;
      }
      let candidate_edges = edges(*candidate);
      let selected_edges = edges(selected);
      Some(json!({
        "index": index,
        "edges": candidate_edges,
        "inside_drag": contains(drag, *candidate),
        "inside_slack": included,
        "iou": intersection_over(drag, *candidate),
        "contributes_to_union": branch == "contained-union" && included && intersection_over(drag, *candidate) > 0.0,
        "matches_selected_edges": (0..4).map(|edge| {
          (candidate_edges[edge] - selected_edges[edge]).abs() < 0.000001
        }).collect::<Vec<_>>(),
      }))
    })
    .collect::<Vec<_>>();
  let trace = json!({
    "time_unix_ms": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok().map(|time| time.as_millis()),
    "coordinate_order": ["left", "top", "right", "bottom"],
    "coordinate_units": "world logical pixels",
    "branch": branch,
    "drag": edges(drag),
    "selected_before_clamp": edges(selected),
    "final_bounds": edges(result),
    "containment_slack": CONTAINMENT_SLACK,
    "total_candidates": boxes.len(),
    "nearby_candidates": candidates,
  });
  let path = std::env::temp_dir().join("screenwide-ruler-snap-trace.json");
  match serde_json::to_vec_pretty(&trace)
    .map_err(|error| error.to_string())
    .and_then(|bytes| std::fs::write(&path, bytes).map_err(|error| error.to_string()))
  {
    Ok(()) => eprintln!("[ruler-snap] {branch}: {}", path.display()),
    Err(error) => eprintln!("[ruler-snap] Could not write trace: {error}"),
  }
}
