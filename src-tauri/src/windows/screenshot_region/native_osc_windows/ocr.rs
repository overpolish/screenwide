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
//! draw call per control with the constant buffer re-pushed between them —
//! exactly what `+ocr_toolbar.m:126-156` did for the crossfading confirm
//! icons.
//!
//! All hit testing, colours, transitions and the two-stage confirm live in the
//! portable `osc::controls` state machines; this file only lays them out,
//! turns them into vertices and forwards activations.

use std::time::Instant;

use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11ShaderResourceView};

use super::renderer::{self, Vertex};
use super::text::TextCache;
use crate::osc::{
  controls::{
    control_metrics, Appearance, ConfirmAction, ConfirmActionSpec, ControlColor, ControlGroup,
    ControlIcon, ControlKind, ControlMetrics, ControlSize, ControlSpec, ControlStyle,
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

/// The status pill: 28pt tall, 8pt radius, 13pt text on a 20pt line box —
/// the `NSTextField` metrics of `+ocr.m:40-66`.
const STATUS_HEIGHT: f64 = 28.0;
const STATUS_RADIUS: f64 = 8.0;
const STATUS_FONT_SIZE: f64 = 13.0;
const STATUS_LINE_HEIGHT: f64 = 20.0;
const STATUS_PADDING_X: f64 = 12.0;
const STATUS_MIN_WIDTH: f64 = 128.0;
const STATUS_MARGIN: f64 = 8.0;
/// The cancel button sits 48pt below the top edge, centred horizontally.
const CANCEL_TOP: f64 = 48.0;
/// OCR's dense 24pt toolbar is squarer than the general compact control.
/// Keep this local so Ruler and editor controls retain their existing shape.
const TOOLBAR_RADIUS: f64 = 6.0;
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

fn cancel_metrics() -> ControlMetrics {
  control_metrics(ControlKind::Button, ControlSize::Default)
}

fn button_metrics() -> ControlMetrics {
  ControlMetrics {
    radius: TOOLBAR_RADIUS,
    ..control_metrics(ControlKind::Button, ControlSize::Compact)
  }
}

fn icon_metrics() -> ControlMetrics {
  ControlMetrics {
    radius: TOOLBAR_RADIUS,
    ..control_metrics(ControlKind::IconButton, ControlSize::Compact)
  }
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

  /// The 2s armed-close timeout (`+ocr_toolbar_input.m:20-34`).
  pub(crate) fn expire_confirm(&mut self) -> ControlOutcome {
    self.expire_confirm_at(Instant::now())
  }

  fn expire_confirm_at(&mut self, now: Instant) -> ControlOutcome {
    if !self.close_armed {
      return ControlOutcome::default();
    }
    let update = self.confirm.expire(now);
    self.close_armed = update.armed;
    ControlOutcome {
      redraw: update.changed,
      animating: update.animating,
      // A platform timer can arrive just before the monotonic deadline. Keep
      // the one-shot alive until ConfirmAction actually reports expiry.
      arm_confirm: update.armed,
      ..Default::default()
    }
  }

  /// Port of `screenwide_region_osc_ocr_control_input` (`+ocr_toolbar_input.m:69`):
  /// the toolbar is offered the event first, then the cancel button.
  pub(crate) fn control_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    let toolbar = self.toolbar_input(point, phase);
    if toolbar.consumed {
      return toolbar;
    }
    let cancel = self.cancel_input(point, phase);
    ControlOutcome {
      redraw: toolbar.redraw || cancel.redraw,
      animating: toolbar.animating || cancel.animating,
      ..cancel
    }
  }

  fn toolbar_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    if !self.toolbar_visible {
      return ControlOutcome::default();
    }
    let update = dispatch_phase(&mut self.toolbar, point, phase);
    let mut outcome = ControlOutcome {
      consumed: update.consumed,
      redraw: update.changed,
      animating: self.toolbar.is_animating(),
      ..Default::default()
    };
    if update.activated == CONTROL_COUNT {
      let confirm = self.confirm.press(Instant::now());
      self.close_armed = confirm.armed;
      outcome.redraw |= confirm.changed;
      outcome.animating |= confirm.animating;
      outcome.arm_confirm = confirm.armed;
      outcome.disarm_confirm = !confirm.armed;
      if confirm.confirmed {
        outcome.dispatch = Some(12);
      }
    } else if update.activated != 0 {
      outcome.dispatch = Some(8 + update.activated as u32);
    }
    outcome
  }

  fn cancel_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    if !self.cancel_visible {
      return ControlOutcome::default();
    }
    let update = dispatch_phase(&mut self.cancel, point, phase);
    ControlOutcome {
      consumed: update.consumed,
      dispatch: (update.activated != 0).then_some(8),
      redraw: update.changed,
      animating: self.cancel.is_animating(),
      ..Default::default()
    }
  }

  /// The world-space half of `screenwide_region_osc_ocr_add_vertices`
  /// (`+ocr.m:100-135`): the 1px selection border during phases 1 and 2, then
  /// one quad per highlight.
  pub(crate) fn add_world_vertices(
    &self,
    out: &mut Vec<Vertex>,
    view: Size,
    region: Rect,
    scale: f64,
  ) {
    if (self.phase == PHASE_LOADING || self.phase == PHASE_READY)
      && region.size.width > 0.0
      && region.size.height > 0.0
    {
      let half = 1.0 / scale.max(1.0);
      let (left, top) = (region.origin.x, region.origin.y);
      let (width, height) = (region.size.width, region.size.height);
      for edge in [
        Rect::from_xywh(left - half, top - half, width + half * 2.0, half * 2.0),
        Rect::from_xywh(
          left - half,
          region.bottom() - half,
          width + half * 2.0,
          half * 2.0,
        ),
        Rect::from_xywh(left - half, top - half, half * 2.0, height + half * 2.0),
        Rect::from_xywh(
          region.right() - half,
          top - half,
          half * 2.0,
          height + half * 2.0,
        ),
      ] {
        renderer::add_quad(out, view, edge, 18);
      }
    }
    for highlight in &self.rects {
      renderer::add_quad(out, view, highlight.rect, rect_kind(highlight.kind));
    }
  }

  /// Lays the chrome out and appends it as one draw call per control. Layout
  /// happens here rather than in `set_ocr` so a resized or re-scaled surface
  /// can never hit-test against a stale rectangle.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add_chrome_vertices(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    region: Rect,
    scale: f64,
    light_mode: bool,
  ) {
    let appearance = if light_mode {
      Appearance::Light
    } else {
      Appearance::Dark
    };
    self.add_status(device, out, segments, view, region, scale, light_mode);
    self.add_cancel(device, out, segments, view, scale, light_mode, appearance);
    self.add_toolbar(
      device, out, segments, view, region, scale, light_mode, appearance,
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn add_status(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    region: Rect,
    scale: f64,
    light_mode: bool,
  ) {
    if !self.status_visible || self.message.is_empty() {
      return;
    }
    let Some(label) = self.text.label(
      device,
      &self.message,
      scale,
      light_mode,
      STATUS_FONT_SIZE,
      STATUS_LINE_HEIGHT,
    ) else {
      return;
    };
    let palette = ocr_palette(light_mode);
    let (fill, foreground, outline) = if self.phase == PHASE_ERROR {
      (
        palette.status_error_fill,
        palette.status_error_foreground,
        palette.error_outline,
      )
    } else {
      (
        palette.loading_fill,
        palette.loading_foreground,
        palette.primary_outline,
      )
    };
    let plate = status_rect(label.size.width, view, region);
    let start = out.len();
    renderer::add_plate(out, view, plate);
    renderer::add_label(
      out,
      view,
      Rect::from_xywh(
        plate.origin.x + (plate.size.width - label.size.width) * 0.5,
        plate.origin.y + (plate.size.height - label.size.height) * 0.5,
        label.size.width,
        label.size.height,
      ),
    );
    push_segment(
      segments,
      out,
      start,
      [fill, foreground],
      STATUS_RADIUS,
      outline,
      Some(label.view.clone()),
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn add_cancel(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    scale: f64,
    light_mode: bool,
    appearance: Appearance,
  ) {
    if !self.cancel_visible {
      return;
    }
    let metrics = cancel_metrics();
    let Some(label) = self.text.label(
      device,
      "Cancel",
      scale,
      light_mode,
      metrics.font_size,
      metrics.line_height,
    ) else {
      return;
    };
    let width = metrics.padding_x * 2.0 + metrics.icon_size + metrics.gap + label.size.width;
    let left = ((view.width - width) * 0.5).floor();
    let plate = Rect::from_xywh(left, CANCEL_TOP, width, metrics.height);
    self.cancel.layout(&[ControlSpec {
      rect: plate,
      style: ControlStyle::button(ControlColor::Neutral, ControlSize::Default),
      icon: ControlIcon::X,
    }]);
    let Some(visual) = self.cancel.visuals(appearance).first().copied() else {
      return;
    };
    let start = out.len();
    add_control(
      out,
      view,
      plate,
      &metrics,
      ControlIcon::X,
      Some(&*label),
      true,
    );
    push_segment(
      segments,
      out,
      start,
      visual_fills(visual),
      metrics.radius,
      [0.0; 4],
      Some(label.view.clone()),
    );
  }

  #[allow(clippy::too_many_arguments)]
  fn add_toolbar(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    region: Rect,
    scale: f64,
    light_mode: bool,
    appearance: Appearance,
  ) {
    if !self.toolbar_visible {
      return;
    }
    let button = button_metrics();
    let icon = icon_metrics();
    let mut labels = Vec::with_capacity(2);
    for text in ["Copy all", "Copy as paragraph"] {
      let Some(label) = self.text.label(
        device,
        text,
        scale,
        light_mode,
        button.font_size,
        button.line_height,
      ) else {
        return;
      };
      labels.push(label);
    }
    let widths = [
      button.padding_x * 2.0 + button.icon_size + button.gap + labels[0].size.width,
      button.padding_x * 2.0 + button.icon_size + button.gap + labels[1].size.width,
      icon.height,
      icon.height,
    ];
    let rects = toolbar::layout(region, view, widths, button.height);
    let specs = std::array::from_fn::<_, CONTROL_COUNT, _>(|index| ControlSpec {
      rect: rects[index],
      style: if index < 2 {
        ControlStyle::button(ControlColor::Neutral, ControlSize::Compact)
      } else {
        ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Compact)
      },
      icon: toolbar_icon(index),
    });
    self.toolbar.layout(&specs);
    let visuals = self.toolbar.visuals(appearance);
    if visuals.len() != CONTROL_COUNT {
      return;
    }
    for index in 0..CONTROL_COUNT {
      let is_button = index < 2;
      let metrics = if is_button { button } else { icon };
      let label = is_button.then(|| &labels[index]);
      let start = out.len();
      add_control(
        out,
        view,
        rects[index],
        &metrics,
        toolbar_icon(index),
        label.map(|texture| &**texture),
        is_button,
      );
      push_segment(
        segments,
        out,
        start,
        visual_fills(visuals[index]),
        metrics.radius,
        [0.0; 4],
        label.map(|label| label.view.clone()),
      );
      // The close button's icon is owned by the confirm state machine, which
      // crossfades two layers. Each layer is its own draw call with a
      // re-pushed foreground, mirroring `+ocr_toolbar.m:126-156`.
      if index == CONTROL_COUNT - 1 {
        self.add_confirm_layers(
          out,
          segments,
          view,
          rects[index],
          &metrics,
          visuals[index],
          appearance,
        );
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn add_confirm_layers(
    &self,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    rect: Rect,
    metrics: &ControlMetrics,
    visual: ControlVisual,
    appearance: Appearance,
  ) {
    for layer in self.confirm.layers(Instant::now(), appearance) {
      if layer.opacity <= 0.002 || layer.scale <= 0.002 {
        continue;
      }
      let size = metrics.icon_size * f64::from(layer.scale);
      let start = out.len();
      renderer::add_icon(
        out,
        view,
        layer.icon as u8,
        rect.origin.x + (rect.size.width - size) * 0.5,
        rect.origin.y + (rect.size.height - size) * 0.5,
        size,
      );
      let mut foreground = layer.foreground;
      foreground[3] *= layer.opacity;
      push_segment(
        segments,
        out,
        start,
        [visual.fill, foreground],
        metrics.radius,
        [0.0; 4],
        None,
      );
    }
  }
}

/// The pill is centred on the selection and kept a margin inside the surface
/// (`+ocr.m:49-57`).
pub(crate) fn status_rect(label_width: f64, view: Size, region: Rect) -> Rect {
  let width = (label_width + STATUS_PADDING_X * 2.0)
    .max(STATUS_MIN_WIDTH)
    .min((view.width - STATUS_MARGIN * 2.0).max(0.0));
  let top = (region.origin.y + region.size.height * 0.5 - STATUS_HEIGHT * 0.5).clamp(
    STATUS_MARGIN,
    (view.height - STATUS_HEIGHT - STATUS_MARGIN).max(STATUS_MARGIN),
  );
  let left = (region.origin.x + region.size.width * 0.5 - width * 0.5).clamp(
    STATUS_MARGIN,
    (view.width - width - STATUS_MARGIN).max(STATUS_MARGIN),
  );
  Rect::from_xywh(left, top, width, STATUS_HEIGHT)
}

/// A plate, its icon and — for text buttons — its label, laid out the way
/// `render_control` (`+ocr_toolbar.m:78-101`) did.
fn add_control(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  metrics: &ControlMetrics,
  icon: ControlIcon,
  label: Option<&super::text::TextTexture>,
  is_button: bool,
) {
  renderer::add_plate(out, view, rect);
  let icon_left = if is_button {
    rect.origin.x + metrics.padding_x
  } else {
    rect.origin.x + (rect.size.width - metrics.icon_size) * 0.5
  };
  renderer::add_icon(
    out,
    view,
    icon as u8,
    icon_left,
    rect.origin.y + (rect.size.height - metrics.icon_size) * 0.5,
    metrics.icon_size,
  );
  if let Some(label) = label {
    renderer::add_label(
      out,
      view,
      Rect::from_xywh(
        rect.origin.x + metrics.padding_x + metrics.icon_size + metrics.gap,
        rect.origin.y + (rect.size.height - label.size.height) * 0.5,
        label.size.width,
        label.size.height,
      ),
    );
  }
}

fn visual_fills(visual: ControlVisual) -> [[f32; 4]; 2] {
  [visual.fill, visual.foreground]
}

fn push_segment(
  segments: &mut Vec<Segment>,
  out: &[Vertex],
  start: usize,
  action_fills: [[f32; 4]; 2],
  radius: f64,
  outline: [f32; 4],
  label: Option<ID3D11ShaderResourceView>,
) {
  if out.len() <= start {
    return;
  }
  segments.push(Segment {
    start: start as u32,
    count: (out.len() - start) as u32,
    action_fills,
    chrome: [radius as f32, MATERIAL_EMPHASIS, 0.0, 0.0],
    chrome_outline: outline,
    label,
    secondary: None,
  });
}

fn dispatch_phase(
  group: &mut ControlGroup,
  point: Point,
  phase: u32,
) -> crate::osc::controls::ControlUpdate {
  match phase {
    PHASE_HOVER | PHASE_DRAG => group.move_to((point.x, point.y)),
    PHASE_DOWN => group.down((point.x, point.y)),
    PHASE_UP => group.up((point.x, point.y)),
    _ => group.clear_hover(),
  }
}

#[cfg(test)]
#[path = "ocr/tests.rs"]
mod tests;
