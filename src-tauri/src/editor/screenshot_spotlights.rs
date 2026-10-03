// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A still's spotlights light the whole picture, not only the layer they are
//! drawn on. Each layer is drawn on its own and the layers are laid together
//! after, so every layer is handed the other layers' spotlights as well as
//! its own: each then shades its own pixels by the same light, and the
//! composite is shaded once, evenly, whichever layer a spotlight is on.
//!
//! The borrowed spotlights go under everything the layer draws itself. Its
//! picture is shaded by them, while its own annotations are not: those keep
//! the place in the shade their own layer gives them. A spotlight's radius
//! and softness are shares of its box, so only its corners have to move from
//! one layer's source pixels into another's.

use super::annotations::{Annotation, AnnotationPoint, AnnotationShape};
use super::ScreenshotWorkspaceOutputSettings;
use crate::screenshots::ScreenshotOutputSettings;

/// Where a layer's source pixels fall on the canvas: the source's top-left
/// corner and how many canvas pixels one source pixel spans. `None` for a
/// layer with no placement to measure, which neither lends nor borrows.
fn source_on_canvas(
  output: &ScreenshotOutputSettings,
  source_width: u32,
) -> Option<(f64, f64, f64)> {
  if !output.has_placement() || source_width == 0 {
    return None;
  }
  let scale = output.image_width / f64::from(source_width);
  (scale.is_finite() && output.image_x.is_finite() && output.image_y.is_finite()).then_some((
    output.image_x,
    output.image_y,
    scale,
  ))
}

impl ScreenshotWorkspaceOutputSettings {
  /// Every spotlight on the layers other than `id`, in workspace order and
  /// moved into `id`'s source pixels: what layer `id` draws under its own
  /// annotations so the spotlights shade it too. `source_width` is each
  /// layer's source width in pixels, by id.
  pub(crate) fn borrowed_spotlights(
    &self,
    id: u64,
    source_width: &impl Fn(u64) -> Option<u32>,
  ) -> Vec<Annotation> {
    let Some((target_x, target_y, target_scale)) = self
      .items
      .iter()
      .find(|item| item.id == id)
      .zip(source_width(id))
      .and_then(|(item, width)| source_on_canvas(&item.output, width))
    else {
      return Vec::new();
    };
    let mut borrowed = Vec::new();
    for item in self.items.iter().filter(|item| item.id != id) {
      let Some((x, y, scale)) =
        source_width(item.id).and_then(|width| source_on_canvas(&item.output, width))
      else {
        continue;
      };
      let into_target = |point: AnnotationPoint| AnnotationPoint {
        x: (x + point.x * scale - target_x) / target_scale,
        y: (y + point.y * scale - target_y) / target_scale,
      };
      borrowed.extend(
        item
          .output
          .annotations
          .iter()
          .filter(|annotation| {
            matches!(annotation.shape, AnnotationShape::Spotlight { .. })
              && annotation.shape.placed()
          })
          .map(|annotation| Annotation {
            shape: annotation.shape.mapped(into_target),
            ..annotation.clone()
          }),
      );
    }
    borrowed
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::spotlight::model::new_spotlight;
  use crate::editor::screenshot_model::ScreenshotWorkspaceItemOutput;
  use crate::screenshots::test_output_settings;

  fn point(x: f64, y: f64) -> AnnotationPoint {
    AnnotationPoint { x, y }
  }

  fn layer(
    id: u64,
    image_x: f64,
    image_width: f64,
    annotations: Vec<Annotation>,
  ) -> ScreenshotWorkspaceItemOutput {
    let mut output = test_output_settings(1_000, 1_000);
    output.image_x = image_x;
    output.image_y = 10.0;
    output.image_width = image_width;
    output.annotations = annotations;
    ScreenshotWorkspaceItemOutput { id, output }
  }

  fn workspace(items: Vec<ScreenshotWorkspaceItemOutput>) -> ScreenshotWorkspaceOutputSettings {
    ScreenshotWorkspaceOutputSettings {
      canvas: test_output_settings(1_000, 1_000),
      items,
    }
  }

  /// Both layers have 400 pixel wide sources. The first is drawn at half
  /// size from x 100, the second at full size from x 300, so a point keeps
  /// its place on the canvas only when it is moved and doubled in scale.
  #[test]
  fn a_spotlight_lights_the_same_place_on_the_canvas_from_every_layer() {
    let light = new_spotlight(
      "s".to_owned(),
      [point(40.0, 20.0), point(240.0, 120.0)],
      None,
    );
    let output = workspace(vec![
      layer(1, 100.0, 200.0, vec![light]),
      layer(2, 300.0, 400.0, Vec::new()),
    ]);
    let borrowed = output.borrowed_spotlights(2, &|_| Some(400));
    let [annotation] = borrowed.as_slice() else {
      panic!("{borrowed:?}");
    };
    // The first corner is at canvas (120, 20); the second layer's source
    // starts at (300, 10), one source pixel to one canvas pixel.
    assert_eq!(
      annotation.shape,
      AnnotationShape::Spotlight {
        start: point(-180.0, 10.0),
        end: point(-80.0, 60.0),
      }
    );
  }

  #[test]
  fn a_layer_borrows_only_the_other_layers_spotlights() {
    let own = new_spotlight("own".to_owned(), [point(0.0, 0.0), point(10.0, 10.0)], None);
    let arrow = crate::editor::annotations::arrow::model::new_arrow(
      "arrow".to_owned(),
      point(0.0, 0.0),
      point(10.0, 10.0),
      None,
    );
    let output = workspace(vec![
      layer(1, 0.0, 400.0, vec![own]),
      layer(2, 0.0, 400.0, vec![arrow]),
    ]);
    let ids = |id| {
      output
        .borrowed_spotlights(id, &|_| Some(400))
        .into_iter()
        .map(|annotation| annotation.id)
        .collect::<Vec<_>>()
    };
    assert_eq!(ids(1), Vec::<String>::new());
    assert_eq!(ids(2), vec!["own".to_owned()]);
  }
}
