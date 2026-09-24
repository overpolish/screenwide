// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) struct SurfaceState {
  /// The published arrow chrome and any drag over it.
  pub(super) annotation: annotation::AnnotationState,
  pub(super) backdrop: [f64; 4],
  pub(super) camera_source: Option<compositor::SourceTexture>,
  pub(super) editor_active: bool,
  /// The immutable workspace state a live Frame resize re-flows from, and the
  /// marker that the native side - not the DOM - owns the pane geometry.
  pub(super) frame_resize: Option<FrameResizeStart>,
  /// A Frame resize has ended and its committed layout has not arrived yet:
  /// that layout keeps the rebased transform instead of restoring one.
  pub(super) frame_resize_committed: bool,
  pub(super) gesture: Option<ActiveGesture>,
  pub(super) last_pointer: (f64, f64),
  /// A live layer Move that may grow its canvas under Alt (the Metal
  /// backend's `selectionMoveTargetsStart` / `selectionMoveAutoFitActive`).
  /// `None` once the move ends or an Alt release commits the grown canvas.
  pub(super) move_auto_fit: Option<MoveAutoFit>,
  pub(super) panes: Vec<Option<Pane>>,
  /// The width the double-click reset fits into: the space beside an open tool
  /// panel while one holds the basis, 0 for the whole viewport. Only the fit
  /// commands move it; a resize or a gesture never does.
  pub(super) panel_fit_width: f64,
  pub(super) primary_composition: Option<ComposedFrame>,
  pub(super) scale: f64,
  pub(super) selection: Option<PreviewSelection>,
  pub(super) selection_visible: bool,
  pub(super) selection_snapping_enabled: bool,
  pub(super) selection_snap_guide_x: Option<snapping::SnapGuide>,
  pub(super) selection_snap_guide_y: Option<snapping::SnapGuide>,
  pub(super) selection_targets: Vec<PreviewSelection>,
  pub(super) viewport: PreviewSurfaceRect,
  /// Recording marker layouts fill the viewport even above one point per
  /// output pixel; screenshot marker layouts stop at native size.
  pub(super) workspace_allows_upscale: bool,
  pub(super) workspace_natural_size: Option<(u32, u32)>,
  pub(super) workspace_transform: WorkspaceTransform,
  pub(super) workspace_transforms: HashMap<(u32, u32), WorkspaceTransform>,
}

#[derive(Clone, Copy)]
pub(super) struct EditorGesture {
  pub(super) edges: u32,
  pub(super) last_delta: (f64, f64),
  pub(super) last_scale: f64,
  pub(super) operation: SelectionGestureOperation,
  pub(super) pane_start: PreviewSurfaceRect,
  pub(super) pointer_start: (f64, f64),
  pub(super) selection_start: PreviewSelection,
  pub(super) keyboard_start: Option<crate::editor::keyboard_effects::KeyboardOverlay>,
}

/// Everything a Frame resize needs to stay reversible and drift-free: the
/// pane rectangles, the workspace transform and the canvas size as they were
/// when the drag began. Every pointer move re-derives the whole workspace
/// from these, exactly as the Metal backend re-derives it from
/// `selectionFramePaneStarts` / `selectionFrameZoomStart`, so a rebased zoom
/// can never feed back into the next move's geometry.
pub(super) struct FrameResizeStart {
  pub(super) natural_size: Option<(u32, u32)>,
  pub(super) pane_rects: Vec<(usize, PreviewSurfaceRect)>,
  pub(super) transform: WorkspaceTransform,
}

/// What an Alt-drag auto-fit derives every sample from. The layer targets are
/// the mouse-down set in mouse-down canvas units: React re-lays the targets
/// out in each grown canvas meanwhile, and re-using those would compound the
/// renormalisation and collapse the selection.
pub(super) struct MoveAutoFit {
  /// Alt was held on the previous sample, so the canvas is currently grown
  /// and a release has to commit it.
  pub(super) active: bool,
  /// The bounds the last auto-fit sample grew the canvas to, in the current
  /// starts' canvas units, so a commit can re-express the starts in the
  /// committed canvas and Alt can grow it again from there.
  pub(super) last_bounds: Option<PreviewSurfaceRect>,
  /// The composed canvas size at mouse-down, in output pixels, so the grown
  /// box snaps outward to whole pixels exactly like the canvas the managers
  /// fit (`fit_workspace_to_items` / `fit_canvas_to_layers`).
  pub(super) natural_size: Option<(f64, f64)>,
  pub(super) targets_start: Vec<PreviewSelection>,
}

#[derive(Clone, Copy)]
#[allow(clippy::large_enum_variant)]
pub(super) enum ActiveGesture {
  Pan {
    pointer_start: (f64, f64),
    transform_start: WorkspaceTransform,
  },
  Selection(EditorGesture),
}

#[derive(Default)]
pub(super) struct EditorCallbacks {
  pub(super) annotation_gesture: Option<AnnotationGestureCallback>,
  pub(super) annotation_hover: Option<AnnotationHoverCallback>,
  pub(super) annotation_text: Option<AnnotationTextCallback>,
  pub(super) context_menu: Option<ContextMenuCallback>,
  pub(super) gesture: Option<SelectionGestureCallback>,
  pub(super) pointer_down: Option<PointerDownCallback>,
  pub(super) selection: Option<SelectionCallback>,
  pub(super) transform: Option<TransformCallback>,
}

pub(super) struct SurfaceInner {
  /// Open `present_batch` guards. While positive, presents park their frames
  /// on the pane and the closing guard publishes every frame and every
  /// pending geometry in one flush (the DirectComposition analogue of the
  /// macOS single-`CATransaction` batch).
  pub(super) batch_depth: AtomicU32,
  /// A visual-tree change that was left for the next batch flush to commit,
  /// such as the hides `finish_layout` defers inside an open batch. A flush
  /// that only presented frames has nothing to commit: swap-chain content
  /// reaches the screen on its own, and a commit-and-wait per flush is a
  /// display tick of latency on every pointer sample of an arrow drag.
  pub(super) commit_pending: std::sync::atomic::AtomicBool,
  pub(super) selection_pending: std::sync::atomic::AtomicBool,
  pub(super) callbacks: Mutex<EditorCallbacks>,
  pub(super) editor: editor::EditorWindow,
  pub(super) gpu: Gpu,
  pub(super) state: Mutex<SurfaceState>,
}
