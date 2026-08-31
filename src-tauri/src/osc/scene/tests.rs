// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::osc::style::OVERLAY_SHADE_OPACITY;

#[test]
fn hidden_scene_preserves_region_for_lifecycle_restore() {
  let region = Rect::from_xywh(12.0, 34.0, 560.0, 320.0);
  let mut scene = RegionScene {
    region,
    visible: true,
    ..RegionScene::default()
  };
  scene.visible = false;
  assert_eq!(scene.region, region);
}

#[test]
fn region_scene_uses_the_shared_capture_overlay_token() {
  let scene = RegionScene::default();
  assert_eq!(scene.overlay.shade, [0.0, 0.0, 0.0, OVERLAY_SHADE_OPACITY]);
}

#[test]
fn snapshot_and_interaction_are_independent_presentation_axes() {
  let scene = RegionScene {
    interaction: RegionInteraction {
      input_enabled: true,
      allow_drawing: false,
      aspect: Some(16.0 / 9.0),
      exclusion_rect: Some(Rect::from_xywh(4.0, 8.0, 100.0, 44.0)),
    },
    snapshot: SnapshotPresentation {
      presented: true,
      composited: true,
    },
    ..RegionScene::default()
  };
  assert!(scene.interaction.input_enabled);
  assert!(!scene.interaction.allow_drawing);
  assert!(scene.snapshot.presented);
  assert!(scene.snapshot.composited);
}

#[test]
fn screenshot_session_rejects_stale_region_and_teardown_scenes() {
  let normal = RegionScene {
    visible: true,
    desktop_presented: true,
    interaction: RegionInteraction {
      input_enabled: true,
      allow_drawing: false,
      ..RegionScene::default().interaction
    },
    ..RegionScene::default()
  }
  .reconcile_owner(RegionSceneOwner::Screenshot);
  assert!(normal.is_none());

  let quick = RegionScene {
    visible: true,
    interaction: RegionInteraction {
      input_enabled: true,
      allow_drawing: true,
      ..RegionScene::default().interaction
    },
    ..RegionScene::default()
  };
  assert!(
    quick
      .reconcile_owner(RegionSceneOwner::Screenshot)
      .unwrap()
      .desktop_presented
  );
  assert!(quick.reconcile_owner(RegionSceneOwner::Normal).is_none());

  let restored = RegionScene {
    visible: true,
    interaction: RegionInteraction {
      input_enabled: true,
      allow_drawing: false,
      ..RegionScene::default().interaction
    },
    ..RegionScene::default()
  }
  .reconcile_owner(RegionSceneOwner::RestoringNormal)
  .unwrap();
  assert!(restored.visible);
  assert!(restored.desktop_presented);

  let dormant = restored
    .reconcile_owner(RegionSceneOwner::DormantNormal)
    .unwrap();
  assert!(!dormant.visible);
  assert!(!dormant.interaction.input_enabled);
  assert!(!dormant.desktop_presented);
}

#[test]
fn dormant_projection_does_not_destroy_the_requested_normal_scene() {
  let requested = RegionScene {
    region: Rect::from_xywh(12.0, 24.0, 640.0, 360.0),
    visible: true,
    interaction: RegionInteraction {
      input_enabled: true,
      allow_drawing: false,
      ..RegionScene::default().interaction
    },
    ..RegionScene::default()
  };
  let mut state = RegionSceneState::default();
  let dormant = state
    .reconcile_request(requested, RegionSceneOwner::DormantNormal)
    .unwrap();
  state.set_presented(dormant);
  assert!(!state.presented().visible);
  assert!(!state.presented().interaction.input_enabled);
  assert_eq!(
    state.normal_presentation(),
    requested.reconcile_owner(RegionSceneOwner::Normal).unwrap()
  );
}
