// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Chrome {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_toolbar(
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
        ControlStyle::button(ControlColor::Neutral, ControlSize::Regular)
      } else {
        ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Regular)
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
        ControlRender {
          metrics: &metrics,
          icon: toolbar_icon(index),
          label: label.map(|texture| &**texture),
          is_button,
          scale,
        },
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
          scale,
        );
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_confirm_layers(
    &self,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    rect: Rect,
    metrics: &ControlMetrics,
    visual: ControlVisual,
    appearance: Appearance,
    scale: f64,
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
        ((rect.origin.x + (rect.size.width - size) * 0.5) * scale).round() / scale,
        ((rect.origin.y + (rect.size.height - size) * 0.5) * scale).round() / scale,
        (size * scale).round().max(1.0) / scale,
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
