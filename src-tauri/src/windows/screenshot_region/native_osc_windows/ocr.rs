// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The OCR overlay: highlight rectangles, the status pill, the cancel button
//! and the four-button ready toolbar. Port of `+ocr.m`, `+ocr_cancel.m`,
//! `+ocr_toolbar.m` and `+ocr_toolbar_input.m`.
//!
//! macOS gave every floating control its own `NSVisualEffectView` with a small
//! `CAMetalLayer` on top. Here all of it folds into the surface's single swap
//! chain (plan decision 10): the chrome is a run of extra quads, and because
//! each control carries its own fill and foreground, the run is split into one
//! draw call per control with the constant buffer re-pushed between them -
//! exactly what `+ocr_toolbar.m:126-156` did for the crossfading confirm
//! icons.
//!
//! All hit testing, colours, transitions and the two-stage confirm live in the
//! portable `osc::controls` state machines; this file only lays them out,
//! turns them into vertices and forwards activations.

#[path = "ocr/drawing.rs"]
mod drawing;
#[path = "ocr/input.rs"]
mod input;
#[path = "ocr/status.rs"]
mod status;
#[path = "ocr/toolbar.rs"]
mod toolbar_draw;
pub(crate) use drawing::status_rect;
#[path = "ocr/vertices.rs"]
mod vertices;

use drawing::{add_control, push_segment, visual_fills, ControlRender};

use std::time::Instant;

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11ShaderResourceView};

use super::renderer::{self, Vertex};
use super::text::TextCache;
use crate::osc::{
  controls::{
    control_metrics, control_spacing, Appearance, ConfirmAction, ConfirmActionSpec, ControlColor,
    ControlGroup, ControlIcon, ControlKind, ControlMetrics, ControlSize, ControlSpec, ControlStyle,
    ControlVisual,
  },
  geometry::{Point, Rect, Size},
  style::ocr_palette,
};
use crate::text_recognition::toolbar::{self, CONTROL_COUNT};
use crate::text_recognition::visual::OcrRectPacket;

/// Pointer phases, mirroring `InputPhase`.
const PHASE_HOVER: u32 = 1;
const PHASE_DOWN: u32 = 2;
const PHASE_DRAG: u32 = 3;
const PHASE_UP: u32 = 4;

/// `VisualPhase` values the compositor distinguishes.
pub(crate) const PHASE_LOADING: u32 = 1;
pub(crate) const PHASE_READY: u32 = 2;
pub(crate) const PHASE_ERROR: u32 = 3;

/// The status pill takes the regular control metrics: `--spacing-control-height`
/// tall, `--radius-control` round, `--spacing-section` of side padding, and the
/// body role for its label - the geometry of `+ocr.m:87-132`. It has no minimum
/// width: the label plus its padding is the pill.
const STATUS_HEIGHT: f64 = control_metrics(ControlKind::Button, ControlSize::Regular).height;
const STATUS_RADIUS: f64 = control_metrics(ControlKind::Button, ControlSize::Regular).radius;
const STATUS_FONT_SIZE: f64 = control_metrics(ControlKind::Button, ControlSize::Regular).font_size;
const STATUS_LINE_HEIGHT: f64 =
  control_metrics(ControlKind::Button, ControlSize::Regular).line_height;
const STATUS_PADDING_X: f64 = control_spacing().section;
const STATUS_MARGIN: f64 = control_spacing().control_inset;
/// The cancel button sits `--spacing-layout` below the top edge, centred
/// horizontally.
const CANCEL_TOP: f64 = control_spacing().layout;
/// OCR controls sit over a much busier, unzoomed desktop than Ruler labels.
/// Request the material-emphasis pass so their backing reads as the same
/// muted plate before the semantic hover/press fill is applied.
const MATERIAL_EMPHASIS: f32 = 1.0;

/// One draw call: a span of the vertex buffer plus the constants it needs.
/// The base scene run carries no control colours, so it uses the defaults.
pub(crate) struct Segment {
  pub start: u32,
  pub count: u32,
  pub action_fills: [[f32; 4]; 2],
  pub chrome: [f32; 4],
  pub chrome_outline: [f32; 4],
  pub label: Option<ID3D11ShaderResourceView>,
  /// Bound at `t1`. Only the ruler's tolerance notice uses it (kinds 15/37).
  pub secondary: Option<ID3D11ShaderResourceView>,
}

impl Segment {
  pub(crate) fn base(start: usize, end: usize) -> Option<Self> {
    (end > start).then(|| Self {
      start: start as u32,
      count: (end - start) as u32,
      action_fills: [[0.0; 4]; 2],
      chrome: [0.0; 4],
      chrome_outline: [0.0; 4],
      label: None,
      secondary: None,
    })
  }
}

/// One OCR highlight in surface-local points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LocalRect {
  pub rect: Rect,
  pub kind: u8,
}

/// `1 Line→17, 2 Qr→18, 3 QrError→19, 4 Selection→20` (`+ocr.m:127-129`).
pub(crate) fn rect_kind(kind: u8) -> u32 {
  match kind {
    4 => 20,
    3 => 19,
    2 => 18,
    _ => 17,
  }
}

/// Icons 2/3/4 for copy-all, copy-as-paragraph and recognize-another-area; the
/// close button's icon comes from the confirm state machine instead.
pub(crate) fn toolbar_icon(index: usize) -> ControlIcon {
  match index {
    0 => ControlIcon::Copy,
    1 => ControlIcon::Pilcrow,
    2 => ControlIcon::RotateCcw,
    _ => ControlIcon::None,
  }
}

fn intersects(rect: Rect, bounds: Size) -> bool {
  rect.size.width > 0.0
    && rect.size.height > 0.0
    && rect.origin.x < bounds.width
    && rect.origin.y < bounds.height
    && rect.right() > 0.0
    && rect.bottom() > 0.0
}

/// Port of the per-surface filter in `screenwide_region_osc_set_ocr`
/// (`+ocr.m:152-159`): translate each desktop-space rect into this surface's
/// own coordinates and keep only the ones that land on it.
pub(crate) fn local_rects(
  packets: &[OcrRectPacket],
  offset: Point,
  bounds: Size,
) -> Vec<LocalRect> {
  packets
    .iter()
    .filter_map(|packet| {
      let rect = Rect::from_xywh(
        packet.x - offset.x,
        packet.y - offset.y,
        packet.width,
        packet.height,
      );
      intersects(rect, bounds).then_some(LocalRect {
        rect,
        kind: packet.kind,
      })
    })
    .collect()
}

/// Port of the target pick in `+ocr.m:160-167`: the surface whose visible part
/// of the selection is largest hosts the pill and the toolbar.
pub(crate) fn overlap_area(region: Rect, bounds: Size) -> f64 {
  let width = region.right().min(bounds.width) - region.origin.x.max(0.0);
  let height = region.bottom().min(bounds.height) - region.origin.y.max(0.0);
  if width <= 0.0 || height <= 0.0 {
    return 0.0;
  }
  width * height
}

use self::button_metrics as cancel_metrics;

fn button_metrics() -> ControlMetrics {
  control_metrics(ControlKind::Button, ControlSize::Regular)
}

fn icon_metrics() -> ControlMetrics {
  control_metrics(ControlKind::IconButton, ControlSize::Regular)
}

/// What a pointer event did to the chrome, so the caller can redraw, retime
/// the animation and dispatch the command outside the surface lock.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlOutcome {
  pub consumed: bool,
  /// The `InputPhase` to send through the runtime, 8..=12.
  pub dispatch: Option<u32>,
  pub redraw: bool,
  pub animating: bool,
  /// The close button just armed; its two-second expiry needs a timer.
  pub arm_confirm: bool,
  pub disarm_confirm: bool,
}

pub(crate) struct Chrome {
  pub phase: u32,
  rects: Vec<LocalRect>,
  message: String,
  status_visible: bool,
  pub cancel_visible: bool,
  toolbar_visible: bool,
  cancel: ControlGroup,
  toolbar: ControlGroup,
  confirm: ConfirmAction,
  close_armed: bool,
  text: TextCache,
}

impl Default for Chrome {
  fn default() -> Self {
    Self {
      phase: 0,
      rects: Vec::new(),
      message: String::new(),
      status_visible: false,
      cancel_visible: false,
      toolbar_visible: false,
      cancel: ControlGroup::default(),
      toolbar: ControlGroup::default(),
      // `{idle_icon 1, armed_icon 5, colors 0/2, timeout 2000ms}`
      // (`+ocr_toolbar.m:267-268`).
      confirm: ConfirmAction::new(ConfirmActionSpec {
        idle_icon: ControlIcon::X,
        armed_icon: ControlIcon::Trash2,
        idle_color: ControlColor::Neutral,
        armed_color: ControlColor::Error,
        timeout: std::time::Duration::from_millis(2000),
      }),
      close_armed: false,
      text: TextCache::default(),
    }
  }
}

impl Chrome {
  /// First half of `set_ocr`: store this surface's share of the rects and
  /// apply the phase's side effects on the cancel button and the confirm
  /// state (`+ocr.m:149-178`).
  pub(crate) fn apply(
    &mut self,
    phase: u32,
    packets: &[OcrRectPacket],
    message: &str,
    offset: Point,
    bounds: Size,
  ) {
    self.phase = phase;
    self.rects = local_rects(packets, offset, bounds);
    self.message = message.to_owned();
    if phase == PHASE_READY && self.cancel_visible {
      self.cancel_visible = false;
      let _ = self.cancel.clear_hover();
    }
    if phase != PHASE_READY {
      self.close_armed = false;
    }
  }

  /// Second half: only the target surface shows the pill and the toolbar.
  pub(crate) fn set_target(&mut self, is_target: bool) {
    self.status_visible = is_target && (self.phase == PHASE_LOADING || self.phase == PHASE_ERROR);
    let toolbar_visible = is_target && self.phase == PHASE_READY;
    if self.toolbar_visible && !toolbar_visible {
      let _ = self.toolbar.clear_hover();
    }
    self.toolbar_visible = toolbar_visible;
  }

  pub(crate) fn set_cancel_visible(&mut self, visible: bool) {
    self.cancel_visible = visible;
    if !visible {
      let _ = self.cancel.clear_hover();
    }
  }

  pub(crate) fn is_animating(&self) -> bool {
    let now = Instant::now();
    (self.cancel_visible && self.cancel.is_animating())
      || (self.toolbar_visible && (self.toolbar.is_animating() || self.confirm.is_animating(now)))
  }
}

#[cfg(test)]
#[path = "ocr/tests.rs"]
mod tests;
