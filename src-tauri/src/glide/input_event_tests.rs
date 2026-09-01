// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The wire shape of the events the detector reads. They live in their own file
//! so the command layer stays inside the source-size limit.

use super::{
  core::{GlideAction, GlideDetection, GlidePhase},
  events::GlideInputEvent,
  fit::{FitRect, GlideFitEvent},
  icon::GlideIconEvent,
};

#[test]
fn native_detection_payload_matches_the_preview_contract() {
  let payload = serde_json::to_value(GlideInputEvent::Detection {
    detection: GlideDetection {
      became_ready: true,
      changed: true,
      pending: Some(GlideAction::Minimize),
      phase: GlidePhase::Ready,
      region: None,
    },
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({
      "type": "detection",
      "detection": {
        "becameReady": true,
        "changed": true,
        "pending": "minimize",
        "phase": "ready",
        "region": serde_json::Value::Null
      }
    })
  );
}

#[test]
fn start_payload_carries_the_session_to_reveal() {
  let payload = serde_json::to_value(GlideInputEvent::Start { session_id: 7 }).unwrap();

  assert_eq!(
    payload,
    serde_json::json!({ "sessionId": 7, "type": "start" })
  );
}

#[test]
fn end_payload_carries_the_anchor_a_lift_commits_at() {
  let payload = serde_json::to_value(GlideInputEvent::End {
    anchor_x: 640.0,
    anchor_y: -12.5,
    cancelled: false,
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({
      "anchorX": 640.0,
      "anchorY": -12.5,
      "cancelled": false,
      "type": "end"
    })
  );
}

#[test]
fn icon_payload_names_the_session_it_belongs_to() {
  let payload = serde_json::to_value(GlideIconEvent {
    session_id: 3,
    icon_path: Some(std::path::PathBuf::from(
      "/tmp/Screenwide/app-com.apple.finder.png",
    )),
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({
      "sessionId": 3,
      "iconPath": "/tmp/Screenwide/app-com.apple.finder.png"
    })
  );
}

#[test]
fn icon_payload_reports_a_miss_as_null() {
  let payload = serde_json::to_value(GlideIconEvent {
    session_id: 9,
    icon_path: None,
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({ "sessionId": 9, "iconPath": serde_json::Value::Null })
  );
}

#[test]
fn fit_payload_carries_the_achieved_frame_in_work_area_fractions() {
  let payload = serde_json::to_value(GlideFitEvent {
    session_id: 4,
    fits: false,
    actual: FitRect {
      x: 0.75,
      y: 0.0,
      width: 0.25,
      height: 1.0,
    },
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({
      "sessionId": 4,
      "fits": false,
      "actual": { "x": 0.75, "y": 0.0, "width": 0.25, "height": 1.0 }
    })
  );
}

#[test]
fn a_fitting_placement_still_reports_its_frame() {
  let payload = serde_json::to_value(GlideFitEvent {
    session_id: 1,
    fits: true,
    actual: FitRect {
      x: 0.0,
      y: 0.5,
      width: 1.0,
      height: 0.5,
    },
  })
  .unwrap();

  assert_eq!(payload["fits"], serde_json::json!(true));
  assert_eq!(
    payload["actual"],
    serde_json::json!({ "x": 0.0, "y": 0.5, "width": 1.0, "height": 0.5 })
  );
}

#[test]
fn end_payload_carries_the_cancellation_esc_ends_on() {
  let payload = serde_json::to_value(GlideInputEvent::End {
    anchor_x: 0.0,
    anchor_y: 0.0,
    cancelled: true,
  })
  .unwrap();

  assert_eq!(
    payload,
    serde_json::json!({
      "anchorX": 0.0,
      "anchorY": 0.0,
      "cancelled": true,
      "type": "end"
    })
  );
}
