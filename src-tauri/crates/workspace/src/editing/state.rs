// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub struct SurfaceState {
  /// The published arrow chrome and any drag over it.
  pub(super) annotation: annotation::AnnotationState,
  pub backdrop: [f64; 4],
  pub editor_active: bool,
  /// A crop window being drawn from a press outside the current one. Set on
  /// the press, and taken when the gesture ends or is cancelled.
  pub crop_draw: Option<CropDrawStart>,
  /// The immutable workspace state a live Frame resize re-flows from, and the
  /// marker that the native side - not the DOM - owns the pane geometry.
  pub frame_resize: Option<FrameResizeStart>,
  /// A Frame resize has ended and its committed layout has not arrived yet:
  /// that layout keeps the rebased transform instead of restoring one.
  pub frame_resize_committed: bool,
  pub gesture: Option<ActiveGesture>,
  pub last_pointer: (f64, f64),
  /// The modifiers held as of the pointer event being handled.
  pub modifiers: Modifiers,
  /// A live layer Move that may grow its canvas under Alt (the Metal
  /// backend's `selectionMoveTargetsStart` / `selectionMoveAutoFitActive`).
  /// `None` once the move ends or an Alt release commits the grown canvas.
  pub move_auto_fit: Option<MoveAutoFit>,
  /// Objects picked in the camera, while they are.
  pub(super) object_picking: object_picking::ObjectPicking,
  pub panes: Vec<Option<Pane>>,
  /// The width the double-click reset fits into: the space beside an open tool
  /// panel while one holds the basis, 0 for the whole viewport. Only the fit
  /// commands move it; a resize or a gesture never does.
  pub panel_fit_width: f64,
  pub scale: f64,
  pub selection: Option<PreviewSelection>,
  pub selection_visible: bool,
  pub selection_snapping_enabled: bool,
  pub(super) selection_snap_guide_x: Option<snapping::SnapGuide>,
  pub(super) selection_snap_guide_y: Option<snapping::SnapGuide>,
  /// Whether a press picks layers the way the select tool does. Off while a
  /// drawing tool is in hand, when the targets only say which picture a fresh
  /// annotation is drawn on.
  pub selection_hit_testing: bool,
  pub selection_targets: Vec<PreviewSelection>,
  /// The rotation gizmo's held ring and the ring the pointer rests on.
  pub(super) turn: turn::TurnState,
  pub viewport: PreviewSurfaceRect,
  /// Recording marker layouts fill the viewport even above one point per
  /// output pixel; screenshot marker layouts stop at native size.
  pub workspace_allows_upscale: bool,
  pub workspace_natural_size: Option<(u32, u32)>,
  pub workspace_transform: WorkspaceTransform,
  pub workspace_transforms: HashMap<(u32, u32), WorkspaceTransform>,
  pub workspace: WorkspaceDraw,
}

impl Default for SurfaceState {
  fn default() -> Self {
    Self {
      annotation: Default::default(),
      backdrop: [0.09, 0.09, 0.10, 1.0],
      crop_draw: None,
      editor_active: false,
      frame_resize: None,
      frame_resize_committed: false,
      move_auto_fit: None,
      object_picking: Default::default(),
      gesture: None,
      last_pointer: (0.0, 0.0),
      modifiers: Modifiers::default(),
      panes: Vec::new(),
      panel_fit_width: 0.0,
      scale: 1.0,
      selection: None,
      selection_visible: true,
      selection_snapping_enabled: false,
      selection_snap_guide_x: None,
      selection_snap_guide_y: None,
      selection_hit_testing: false,
      selection_targets: Vec::new(),
      turn: Default::default(),
      viewport: PreviewSurfaceRect {
        height: 0.0,
        width: 0.0,
        x: 0.0,
        y: 0.0,
      },
      workspace_allows_upscale: false,
      workspace_transform: WorkspaceTransform::default(),
      workspace_natural_size: None,
      workspace_transforms: HashMap::new(),
      workspace: Default::default(),
    }
  }
}

#[derive(Clone, Copy)]
pub struct EditorGesture {
  pub edges: u32,
  pub last_delta: (f64, f64),
  pub last_scale: f64,
  pub operation: SelectionGestureOperation,
  pub pane_start: PreviewSurfaceRect,
  pub pointer_start: (f64, f64),
  pub selection_start: PreviewSelection,
}

/// Everything a Frame resize needs to stay reversible and drift-free: the
/// pane rectangles, the workspace transform and the canvas size as they were
/// when the drag began. Every pointer move re-derives the whole workspace
/// from these, exactly as the macOS surface re-derives it from
/// `selectionFramePaneStarts` / `selectionFrameZoomStart`, so a rebased zoom
/// can never feed back into the next move's geometry.
pub struct FrameResizeStart {
  pub natural_size: Option<(u32, u32)>,
  pub pane_rects: Vec<(usize, PreviewSurfaceRect)>,
  pub transform: WorkspaceTransform,
}

/// What an Alt-drag auto-fit derives every sample from. The layer targets are
/// the mouse-down set in mouse-down canvas units: React re-lays the targets
/// out in each grown canvas meanwhile, and re-using those would compound the
/// renormalisation and collapse the selection.
pub struct MoveAutoFit {
  /// Alt was held on the previous sample, so the canvas is currently grown
  /// and a release has to commit it.
  pub active: bool,
  /// The bounds the last auto-fit sample grew the canvas to, in the current
  /// starts' canvas units, so a commit can re-express the starts in the
  /// committed canvas and Alt can grow it again from there.
  pub last_bounds: Option<PreviewSurfaceRect>,
  /// The composed canvas size at mouse-down, in output pixels, so the grown
  /// box snaps outward to whole pixels exactly like the canvas the managers
  /// fit (`fit_workspace_to_items` / `fit_canvas_to_layers`).
  pub natural_size: Option<(f64, f64)>,
  pub targets_start: Vec<PreviewSelection>,
}

/// Where a drawn crop window is anchored, in pane fractions, and whether the
/// press has travelled far enough to begin the gesture. A press that never
/// does is a plain click and leaves the crop alone.
#[derive(Clone, Copy)]
pub struct CropDrawStart {
  pub anchor: (f64, f64),
  pub begun: bool,
}

#[derive(Clone, Copy)]
#[allow(clippy::large_enum_variant)]
pub enum ActiveGesture {
  Pan {
    pointer_start: (f64, f64),
    transform_start: WorkspaceTransform,
  },
  Selection(EditorGesture),
}

#[derive(Default)]
pub struct EditorCallbacks {
  pub annotation_gesture: Option<AnnotationGestureCallback>,
  pub annotation_hover: Option<AnnotationHoverCallback>,
  pub annotation_text: Option<AnnotationTextCallback>,
  pub context_menu: Option<ContextMenuCallback>,
  pub gesture: Option<SelectionGestureCallback>,
  pub pointer_down: Option<PointerDownCallback>,
  pub selection: Option<SelectionCallback>,
  pub object_pick: Option<screenwide_compositor::object_picking::ObjectPickCallback>,
  pub transform: Option<TransformCallback>,
}

pub struct SurfaceInner {
  /// Open `present_batch` guards. While positive, presents park their frames
  /// on the pane and the closing guard publishes every frame and every
  /// pending geometry in one flush (the analogue of the macOS
  /// single-`CATransaction` batch).
  pub batch_depth: AtomicU32,
  /// A change to what is on screen that was left for the next batch flush to
  /// commit, such as the hides `finish_layout` defers inside an open batch. A
  /// flush that only presented frames has nothing to commit: drawn frames
  /// reach the screen on their own, and a commit-and-wait per flush is a
  /// display tick of latency on every pointer sample of an arrow drag.
  pub commit_pending: std::sync::atomic::AtomicBool,
  pub selection_pending: std::sync::atomic::AtomicBool,
  pub callbacks: Mutex<EditorCallbacks>,
  pub state: Mutex<SurfaceState>,
  pub host: Box<dyn Host>,
}

impl SurfaceInner {
  pub fn new(host: Box<dyn Host>, state: SurfaceState) -> Self {
    Self {
      batch_depth: AtomicU32::new(0),
      commit_pending: false.into(),
      selection_pending: false.into(),
      callbacks: Mutex::default(),
      state: Mutex::new(state),
      host,
    }
  }
}
