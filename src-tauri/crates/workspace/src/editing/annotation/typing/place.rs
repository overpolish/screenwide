// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where what is typed into sets its lines on screen: a box where its grips
//! place it, a caption where its pane last laid it out.

use super::*;
use screenwide_compositor::annotation_gpu::caption_editing_block;
use screenwide_text::engine::metrics::line_width;
use screenwide_text::engine::typing::{TypingLayout, TypingLines};

/// Where a box sets its lines on screen, in display points.
pub(super) struct Place {
  pub(super) left: f64,
  pub(super) top: f64,
  pub(super) width: f64,
  pub(super) height: f64,
  pub(super) font: f64,
  pub(super) align: u32,
  pub(super) face: TypeFace,
}

impl Place {
  /// `work` done with the box's lines as they are drawn: by `words` where
  /// they set chosen words apart, otherwise set in the place's face.
  pub(super) fn layout<R>(
    &self,
    words: Option<&TypedWords>,
    work: impl FnOnce(&mut dyn TypingLines) -> R,
  ) -> R {
    if let Some(words) = words.filter(|words| words.sets_apart()) {
      return words.with_lines(self.font, self.align, |lines| work(lines));
    }
    let width = |line: &str| line_width(&self.face, line, self.font);
    work(&mut TypingLayout {
      width: &width,
      block_width: self.width,
      font: self.font,
      align: self.align,
    })
  }

  pub(super) fn local(&self, point: (f64, f64)) -> (f64, f64) {
    (point.0 - self.left, point.1 - self.top)
  }

  pub(super) fn contains(&self, point: (f64, f64)) -> bool {
    let (x, y) = self.local(point);
    (0.0..=self.width).contains(&x) && (0.0..=self.height).contains(&y)
  }
}

pub(super) fn place(state: &SurfaceState, target: Target, face: &TypeFace) -> Option<Place> {
  let flat = match target {
    Target::Box(flat) => flat,
    Target::Caption(caption) => return caption_place(state, caption, face),
  };
  let item = state
    .annotation
    .handles
    .get(flat)
    .filter(|item| item.kind == AnnotationKind::Text.raw())?;
  let image = item_image_frame(state, flat as i32)?;
  let centre = text_geometry(image, item).start_head.c;
  let (width, height) = (item.middle_x * image.width, item.middle_y * image.width);
  Some(Place {
    left: f64::from(centre[0]) - width / 2.0,
    top: f64::from(centre[1]) - height / 2.0,
    width,
    height,
    font: item.width * image.width,
    align: item.start_head as u32 & 3,
    face: face.clone(),
  })
}

/// Where the caption being typed into on the pane `caption` names sets its
/// words, as its pane last laid them out.
fn caption_place(
  state: &SurfaceState,
  mut caption: PreviewSelection,
  face: &TypeFace,
) -> Option<Place> {
  let pane = state.panes.get(caption.pane_index as usize)?.as_ref()?;
  let settings = pane.settings.as_ref()?;
  let (track, _) = pane.captions.as_ref()?;
  let (block, size, align) = caption_editing_block(track, (settings.width, settings.height))?;
  caption.layer_id = CAPTIONS_LAYER_ID;
  [caption.x, caption.y, caption.width, caption.height] = block;
  let rect = display_selection(state, caption)?;
  Some(Place {
    left: rect.x,
    top: rect.y,
    width: rect.width,
    height: rect.height,
    font: rect.height * size,
    align: (align.clamp(0.0, 1.0) * 2.0).round() as u32,
    face: face.clone(),
  })
}
