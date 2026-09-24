// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::PreviewSelection;
use super::native_types::{NativeWorkspaceLayer, NativeWorkspacePaneRect};
use crate::editor::annotations::handles::{NativeAnnotationHandles, NativeAnnotationSnap};
use crate::editor::cursor_effects::NativeGpuArtwork;
use crate::screenshots::{NativeCanvas, StillOverlay};

unsafe extern "C" {
  pub(super) fn screenwide_preview_surface_create(
    host_view: *mut std::ffi::c_void,
  ) -> *mut std::ffi::c_void;
  pub(super) fn screenwide_preview_surface_layout_workspace(
    handle: *mut std::ffi::c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    natural_width: f64,
    natural_height: f64,
    defer_draw: i32,
  );
  pub(super) fn screenwide_preview_surface_layout_recording_workspace(
    handle: *mut std::ffi::c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    natural_width: f64,
    natural_height: f64,
    panes: *const NativeWorkspacePaneRect,
    pane_count: u32,
    defer_draw: i32,
  );
  pub(super) fn screenwide_preview_surface_set_viewport(
    handle: *mut std::ffi::c_void,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
  );
  pub(super) fn screenwide_preview_surface_begin_layout(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_finish_layout(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_clear_workspace_transform_history(
    handle: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_begin_present(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_end_present(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_present(
    handle: *mut std::ffi::c_void,
    index: u32,
    rgba: *const u8,
    width: u32,
    height: u32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_present_screenshot_workspace(
    handle: *mut std::ffi::c_void,
    layers: *const NativeWorkspaceLayer,
    layer_count: u32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_present_recording_workspace(
    handle: *mut std::ffi::c_void,
    layers: *const NativeWorkspaceLayer,
    layer_count: u32,
    artworks: *const NativeGpuArtwork,
    artwork_count: u32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_workspace_source_size(
    handle: *mut std::ffi::c_void,
    pane_index: u32,
    width: *mut u32,
    height: *mut u32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_workspace_camera_source_size(
    handle: *mut std::ffi::c_void,
    pane_index: u32,
    width: *mut u32,
    height: *mut u32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_update_workspace_canvas(
    handle: *mut std::ffi::c_void,
    pane_index: u32,
    canvas_width: u32,
    canvas_height: u32,
    canvas: *const NativeCanvas,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_update_workspace_camera_overlay(
    handle: *mut std::ffi::c_void,
    pane_index: u32,
    overlay: *const StillOverlay,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_set_workspace_annotation_hover(
    handle: *mut std::ffi::c_void,
    pane_index: u32,
    index: i32,
    width: f32,
  ) -> i32;
  pub(super) fn screenwide_preview_surface_redraw_workspace(handle: *mut std::ffi::c_void) -> i32;
  pub(super) fn screenwide_preview_surface_hide(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_destroy(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_enable_editor(
    handle: *mut std::ffi::c_void,
    callback: Option<unsafe extern "C" fn(f64, *mut std::ffi::c_void)>,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_editor_suspended(
    handle: *mut std::ffi::c_void,
    suspended: i32,
  );
  pub(super) fn screenwide_preview_surface_set_editor_zoom(
    handle: *mut std::ffi::c_void,
    zoom_percent: f64,
  );
  pub(super) fn screenwide_preview_surface_reset_editor_view(
    handle: *mut std::ffi::c_void,
    fit_width: f64,
  );
  pub(super) fn screenwide_preview_surface_set_editor_fit_basis(
    handle: *mut std::ffi::c_void,
    fit_width: f64,
  );
  pub(super) fn screenwide_preview_surface_set_selection_visible(
    handle: *mut std::ffi::c_void,
    visible: i32,
  );
  pub(super) fn screenwide_preview_surface_set_selection(
    handle: *mut std::ffi::c_void,
    selection: *const PreviewSelection,
  );
  pub(super) fn screenwide_preview_surface_set_selection_targets(
    handle: *mut std::ffi::c_void,
    targets: *const PreviewSelection,
    count: usize,
    enabled: i32,
  );
  pub(super) fn screenwide_preview_surface_set_selection_snapping(
    handle: *mut std::ffi::c_void,
    enabled: i32,
  );
  pub(super) fn screenwide_preview_surface_set_selection_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<unsafe extern "C" fn(i32, *mut std::ffi::c_void)>,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_pointer_down_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_context_menu_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<unsafe extern "C" fn(u32, f64, f64, *mut std::ffi::c_void)>,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_selection_gesture_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<
      unsafe extern "C" fn(u32, u32, u32, u32, f64, f64, f64, *mut std::ffi::c_void),
    >,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_annotations(
    handle: *mut std::ffi::c_void,
    handles: *const NativeAnnotationHandles,
    count: usize,
    selected_index: i32,
    mode: i32,
    active_layer: i32,
  );
  pub(super) fn screenwide_preview_surface_set_annotation_hover_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<unsafe extern "C" fn(i32, f64, f64, *mut std::ffi::c_void)>,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_annotation_snap(
    handle: *mut std::ffi::c_void,
    snap: *const NativeAnnotationSnap,
  );
  pub(super) fn screenwide_preview_surface_set_annotation_gesture_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<
      unsafe extern "C" fn(u32, u32, u32, u32, u32, f64, f64, u32, f64, *mut std::ffi::c_void),
    >,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_set_annotation_text_callback(
    handle: *mut std::ffi::c_void,
    callback: Option<
      unsafe extern "C" fn(u32, u32, u32, *const u8, u32, u64, *mut std::ffi::c_void),
    >,
    context: *mut std::ffi::c_void,
  );
  pub(super) fn screenwide_preview_surface_begin_annotation_text(
    handle: *mut std::ffi::c_void,
    index: i32,
    text: *const u8,
    length: u32,
    dark_ink: u32,
  );
  pub(super) fn screenwide_preview_surface_end_annotation_text(handle: *mut std::ffi::c_void);
  pub(super) fn screenwide_preview_surface_set_audio_ribbon(
    handle: *mut std::ffi::c_void,
    samples: *const f32,
    track_count: u32,
    point_count: u32,
    gains: *const f32,
  );
  pub(super) fn screenwide_preview_surface_set_audio_ribbon_clock(
    handle: *mut std::ffi::c_void,
    context: *mut std::ffi::c_void,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, f64) -> f64,
    release: unsafe extern "C" fn(*mut std::ffi::c_void),
  );
  pub(super) fn screenwide_preview_surface_set_audio_ribbon_playhead(
    handle: *mut std::ffi::c_void,
    ratio: f64,
  );
  pub(super) fn screenwide_preview_surface_release_context_on_main(
    release: unsafe extern "C" fn(*mut std::ffi::c_void),
    context: *mut std::ffi::c_void,
  );
}
