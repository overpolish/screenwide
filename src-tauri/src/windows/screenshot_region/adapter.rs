// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compile-time platform boundary for the Region OSC host.
//!
//! Tool workflow code submits portable scene and desktop data here. Platform
//! implementations own native window handles, compositor surfaces and GPU
//! submission, but never Region interaction or lifecycle policy.

use crate::osc::{
  desktop::DesktopBinding,
  geometry::Rect,
  scene::RegionScene,
  semantic::{SemanticEvent, SemanticRegion, SemanticStatus},
};

#[cfg(target_os = "macos")]
#[path = "adapter/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "adapter/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "adapter/unavailable.rs"]
mod platform;

#[cfg(target_os = "windows")]
pub(super) use platform::set_capture_affinity;
pub(super) use platform::{
  acquire_quick_screenshot_cursor, apply_region_scene, prepare_for_region_restore,
  prepare_for_screenshot, release_quick_screenshot_cursor, set_desktop_presented,
  set_frame_visible, set_magnifier_source,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RegionSceneRequest {
  pub rect: Rect,
  pub visible: bool,
  pub aspect: Option<f64>,
  pub input_enabled: bool,
  pub exclusion_rect: Option<Rect>,
  pub show_frame: bool,
  pub show_handles: bool,
  pub allow_drawing: bool,
  pub monitor_width: f64,
  pub monitor_height: f64,
  pub desktop_anchor: Option<u32>,
}

pub(super) struct RegionSceneResolution {
  pub scene: RegionScene,
  pub controller_committed: Option<Rect>,
  pub layout_event: Option<SemanticEvent>,
}

pub(super) fn resolve_region_scene(
  request: RegionSceneRequest,
  mut scene: RegionScene,
  desktop: Option<&DesktopBinding>,
) -> RegionSceneResolution {
  let committed = request.rect.committed().then_some(request.rect);
  let reconciled =
    desktop.and_then(|binding| committed.and_then(|region| binding.reconcile_local(region)));
  scene.region = reconciled
    .map(|region| region.global)
    .or_else(|| desktop.and_then(|binding| binding.project_local(request.rect)))
    .unwrap_or(request.rect);
  scene.visible = request.visible;
  scene.chrome.frame_visible = request.show_frame;
  scene.chrome.handles_visible = request.show_handles;
  scene.interaction.input_enabled = request.input_enabled;
  scene.interaction.allow_drawing = request.allow_drawing;
  scene.interaction.aspect = request.aspect;
  scene.interaction.exclusion_rect = request.exclusion_rect;
  let controller_committed = desktop
    .map(|_| reconciled.map(|region| region.anchor_local))
    .unwrap_or(committed);
  let layout_event = desktop.and_then(|binding| {
    binding.layout_changed.then(|| SemanticEvent {
      status: SemanticStatus::Layout,
      gesture: None,
      region: reconciled.map(|region| SemanticRegion {
        x: region.owner_local.origin.x,
        y: region.owner_local.origin.y,
        width: region.owner_local.size.width,
        height: region.owner_local.size.height,
      }),
      monitor_id: Some(reconciled.map_or(binding.anchor_id, |region| region.owner_id)),
    })
  });
  RegionSceneResolution {
    scene,
    controller_committed,
    layout_event,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::osc::geometry::{Point, Size};

  #[test]
  fn region_adapter_request_preserves_cross_display_geometry_and_policy() {
    let rect = Rect {
      origin: Point { x: 1700.0, y: 40.0 },
      size: Size {
        width: 500.0,
        height: 300.0,
      },
    };
    let request = RegionSceneRequest {
      rect,
      visible: true,
      aspect: None,
      input_enabled: true,
      exclusion_rect: None,
      show_frame: true,
      show_handles: false,
      allow_drawing: true,
      monitor_width: 3720.0,
      monitor_height: 1169.0,
      desktop_anchor: Some(1),
    };

    assert_eq!(request.rect, rect);
    assert_eq!(request.desktop_anchor, Some(1));
    assert!(request.visible && request.input_enabled && request.allow_drawing);
  }

  #[test]
  fn shared_resolution_projects_scene_and_layout_to_the_new_owner() {
    use crate::osc::desktop::DesktopDisplay;

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
      layout_changed: true,
    };
    let request = RegionSceneRequest {
      rect: Rect::from_xywh(90.0, 10.0, 80.0, 30.0),
      visible: true,
      aspect: None,
      input_enabled: true,
      exclusion_rect: None,
      show_frame: true,
      show_handles: true,
      allow_drawing: false,
      monitor_width: 220.0,
      monitor_height: 80.0,
      desktop_anchor: Some(1),
    };

    let resolved = resolve_region_scene(request, RegionScene::default(), Some(&binding));

    assert_eq!(
      resolved.scene.region,
      Rect::from_xywh(90.0, 10.0, 80.0, 30.0)
    );
    assert_eq!(resolved.controller_committed, Some(request.rect));
    assert_eq!(resolved.layout_event.unwrap().monitor_id, Some(2));
  }

  #[test]
  fn selected_platform_satisfies_the_region_surface_contract() {
    let _: fn(&tauri::AppHandle, tauri::WebviewWindow, RegionSceneRequest) -> Result<bool, String> =
      platform::apply_region_scene;
    let _: fn(&tauri::WebviewWindow, bool) -> tauri::Result<()> = platform::set_desktop_presented;
    let _: fn(&tauri::WebviewWindow) -> tauri::Result<()> = platform::prepare_for_screenshot;
    let _: fn(&tauri::WebviewWindow) -> tauri::Result<()> = platform::prepare_for_region_restore;
    let _: fn(&tauri::WebviewWindow, bool) -> Result<bool, String> = platform::set_frame_visible;
  }
}
