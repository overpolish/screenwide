// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A still's layers stack in workspace order, each with its annotations over
//! its picture, and the spotlights lay one shade over that stack, at the
//! topmost of them: it darkens everything under it and nothing above, and
//! every spotlight cuts its hole in it, whichever layer it is on. Each layer
//! is drawn on its own and the layers are laid together after, so a layer
//! the shade reaches is handed the other layers' spotlights as well as its
//! own: under everything it draws when its own spotlight is the topmost, so
//! they only cut their holes, and over everything when a higher layer holds
//! the shade, so all of it lies in the shade. A layer above the topmost
//! spotlight borrows none and stays bright. A spotlight's radius and
//! softness are shares of its box, so only its corners have to move from one
//! layer's source pixels into another's.

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

/// The spotlights a layer draws from the other layers, in workspace order and
/// moved into its source pixels, and whether they go over its own annotations
/// rather than under them.
pub(crate) struct BorrowedSpotlights {
  pub(crate) spotlights: Vec<Annotation>,
  pub(crate) over: bool,
}

impl BorrowedSpotlights {
  /// How far the borrowed spotlights move the layer's own annotations up its
  /// drawn list.
  pub(crate) fn shift(&self) -> usize {
    if self.over {
      0
    } else {
      self.spotlights.len()
    }
  }

  /// Lays the borrowed spotlights into the layer's own `annotations`.
  pub(crate) fn lay_into(self, annotations: &mut Vec<Annotation>) {
    if self.over {
      annotations.extend(self.spotlights);
    } else {
      annotations.splice(0..0, self.spotlights);
    }
  }
}

fn lights(annotation: &Annotation) -> bool {
  matches!(annotation.shape, AnnotationShape::Spotlight { .. }) && annotation.shape.placed()
}

impl ScreenshotWorkspaceOutputSettings {
  /// The other layers' spotlights layer `id` draws so the one shade lies
  /// over the stack as the module describes. `source_width` is each layer's
  /// source width in pixels, by id.
  pub(crate) fn borrowed_spotlights(
    &self,
    id: u64,
    source_width: &impl Fn(u64) -> Option<u32>,
  ) -> BorrowedSpotlights {
    let none = || BorrowedSpotlights {
      spotlights: Vec::new(),
      over: false,
    };
    let Some(place) = self.items.iter().position(|item| item.id == id) else {
      return none();
    };
    let Some(shade) = self
      .items
      .iter()
      .rposition(|item| item.output.annotations.iter().any(lights))
    else {
      return none();
    };
    if shade < place {
      return none();
    }
    let Some((target_x, target_y, target_scale)) =
      source_width(id).and_then(|width| source_on_canvas(&self.items[place].output, width))
    else {
      return none();
    };
    let mut spotlights = Vec::new();
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
      spotlights.extend(
        item
          .output
          .annotations
          .iter()
          .filter(|annotation| lights(annotation))
          .map(|annotation| Annotation {
            shape: annotation.shape.mapped(into_target),
            ..annotation.clone()
          }),
      );
    }
    BorrowedSpotlights {
      spotlights,
      over: shade > place,
    }
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

  /// Both layers have 400 pixel wide sources. The first is drawn at full
  /// size from x 300, the second at half size from x 100, so a point keeps
  /// its place on the canvas only when it is moved and doubled in scale.
  #[test]
  fn a_spotlight_lights_the_same_place_on_the_canvas_from_every_layer() {
    let light = new_spotlight(
      "s".to_owned(),
      [point(40.0, 20.0), point(240.0, 120.0)],
      None,
    );
    let output = workspace(vec![
      layer(1, 300.0, 400.0, Vec::new()),
      layer(2, 100.0, 200.0, vec![light]),
    ]);
    let borrowed = output.borrowed_spotlights(1, &|_| Some(400));
    let [annotation] = borrowed.spotlights.as_slice() else {
      panic!("{:?}", borrowed.spotlights);
    };
    // The first corner is at canvas (120, 20); the first layer's source
    // starts at (300, 10), one source pixel to one canvas pixel.
    assert_eq!(
      annotation.shape,
      AnnotationShape::Spotlight {
        start: point(-180.0, 10.0),
        end: point(-80.0, 60.0),
      }
    );
  }

  /// Three layers, the middle one holding the topmost spotlight: the layer
  /// under it lies wholly in the shade, the middle one takes the lower
  /// spotlight's hole under its own, and the layer above stays bright.
  #[test]
  fn the_shade_darkens_the_layers_under_the_topmost_spotlight_only() {
    let spotlight =
      |id: &str| new_spotlight(id.to_owned(), [point(0.0, 0.0), point(10.0, 10.0)], None);
    let arrow = crate::editor::annotations::arrow::model::new_arrow(
      "arrow".to_owned(),
      point(0.0, 0.0),
      point(10.0, 10.0),
      None,
    );
    let output = workspace(vec![
      layer(1, 0.0, 400.0, vec![spotlight("low")]),
      layer(2, 0.0, 400.0, vec![spotlight("high")]),
      layer(3, 0.0, 400.0, vec![arrow]),
    ]);
    let borrowed = |id| {
      let borrowed = output.borrowed_spotlights(id, &|_| Some(400));
      let ids = borrowed
        .spotlights
        .iter()
        .map(|annotation| annotation.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
      (ids, borrowed.over)
    };
    assert_eq!(borrowed(1), ("high".to_owned(), true));
    assert_eq!(borrowed(2), ("low".to_owned(), false));
    assert_eq!(borrowed(3), (String::new(), false));
  }
}
