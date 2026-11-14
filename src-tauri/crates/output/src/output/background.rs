// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas's background as one value, and the one a scene's background
//! gives way to while it blooms in over it.

use screenwide_background::{Background, BackgroundMeshPoint, DEFAULT_GENERATOR};

use super::{MeshGradientPoint, ScreenshotOutputSettings};

/// A background in the flat fields a canvas draws it from: the one giving
/// way while a scene's blooms in over it.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasBackground {
  pub background_type: String,
  pub background_color: String,
  pub background_image_path: Option<String>,
  pub mesh_colors: Vec<String>,
  pub mesh_generator: String,
  pub mesh_points: Vec<MeshGradientPoint>,
  pub mesh_seed: u32,
  pub mesh_warp_percent: f64,
}

/// A scene's background blooming in over the one before it: the incoming
/// one is the canvas's own, spreading from where the incoming screen lands.
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundTurn {
  /// The background giving way, drawn outside the bloom.
  pub outgoing: CanvasBackground,
  /// How far the bloom has spread, eased, from nothing to the whole canvas.
  pub share: f64,
  /// Where it spreads from, in shares of the canvas.
  pub centre: (f64, f64),
  /// How much larger the backgrounds are drawn by the time it has spread,
  /// as a share: positive where the screen grows, as if the camera moved
  /// closer, negative where it shrinks; zero for a move alone.
  pub push: f64,
}

impl BackgroundTurn {
  /// How large the outgoing and the incoming backgrounds are drawn now,
  /// about `centre`. Both move as one camera does: the outgoing one leaves
  /// at its own size and the incoming one lands at its own.
  pub fn scales(&self) -> (f64, f64) {
    let push = self.push.abs();
    let reached = 1.0 + push * self.share.clamp(0.0, 1.0);
    if self.push >= 0.0 {
      (reached, reached / (1.0 + push))
    } else {
      (1.0 / reached, (1.0 + push) / reached)
    }
  }
}

/// The flat fields a background is spread across, borrowed from a canvas or
/// from the background giving way on it.
struct Fields<'a> {
  kind: &'a mut String,
  color: &'a mut String,
  picture: &'a mut Option<String>,
  colors: &'a mut Vec<String>,
  generator: &'a mut String,
  points: &'a mut Vec<MeshGradientPoint>,
  seed: &'a mut u32,
  warp_percent: &'a mut f64,
}

/// Puts `background` into `fields`. Only the chosen kind's own fields are
/// written, so the other kinds keep theirs, as the webview's
/// `applyBackgroundToOutput` does; a composition saved without its blobs
/// keeps the ones the canvas has.
fn wear(fields: Fields<'_>, background: &Background) {
  match background {
    Background::Solid { color } => {
      "solid".clone_into(fields.kind);
      color.clone_into(fields.color);
    }
    Background::Image { path } => {
      "image".clone_into(fields.kind);
      *fields.picture = Some(path.clone());
    }
    Background::Mesh {
      colors,
      seed,
      warp_percent,
      generator,
      points,
      ..
    } => {
      "mesh".clone_into(fields.kind);
      colors.clone_into(fields.colors);
      generator.clone_into(fields.generator);
      if !points.is_empty() {
        *fields.points = points.iter().map(point).collect();
      }
      *fields.seed = *seed;
      *fields.warp_percent = *warp_percent;
    }
  }
}

fn point(point: &BackgroundMeshPoint) -> MeshGradientPoint {
  MeshGradientPoint {
    radius_x: point.radius_x,
    radius_y: point.radius_y,
    rotation: point.rotation,
    x: point.x,
    y: point.y,
  }
}

impl CanvasBackground {
  /// `background` as `canvas` would draw it, the fields its kind leaves
  /// alone taken from the canvas.
  pub fn worn_on(canvas: &ScreenshotOutputSettings, background: &Background) -> Self {
    let mut worn = Self {
      background_type: canvas.background_type.clone(),
      background_color: canvas.background_color.clone(),
      background_image_path: canvas.background_image_path.clone(),
      mesh_colors: canvas.mesh_colors.clone(),
      mesh_generator: canvas.mesh_generator.clone(),
      mesh_points: canvas.mesh_points.clone(),
      mesh_seed: canvas.mesh_seed,
      mesh_warp_percent: canvas.mesh_warp_percent,
    };
    wear(
      Fields {
        kind: &mut worn.background_type,
        color: &mut worn.background_color,
        picture: &mut worn.background_image_path,
        colors: &mut worn.mesh_colors,
        generator: &mut worn.mesh_generator,
        points: &mut worn.mesh_points,
        seed: &mut worn.mesh_seed,
        warp_percent: &mut worn.mesh_warp_percent,
      },
      background,
    );
    worn
  }
}

impl ScreenshotOutputSettings {
  /// The background the canvas draws, as a preset or a scene keeps it.
  pub fn background(&self) -> Background {
    match self.background_type.as_str() {
      "image" => Background::Image {
        path: self.background_image_path.clone().unwrap_or_default(),
      },
      "mesh" => Background::Mesh {
        colors: self.mesh_colors.clone(),
        seed: self.mesh_seed,
        warp_percent: self.mesh_warp_percent,
        generator: if self.mesh_generator.is_empty() {
          DEFAULT_GENERATOR.to_owned()
        } else {
          self.mesh_generator.clone()
        },
        locked_colors: self.mesh_locked_colors.clone(),
        points: self
          .mesh_points
          .iter()
          .map(|point| BackgroundMeshPoint {
            radius_x: point.radius_x,
            radius_y: point.radius_y,
            rotation: point.rotation,
            x: point.x,
            y: point.y,
          })
          .collect(),
      },
      _ => Background::Solid {
        color: self.background_color.clone(),
      },
    }
  }

  /// Draws `background` behind the canvas from now on.
  pub fn wear_background(&mut self, background: &Background) {
    wear(
      Fields {
        kind: &mut self.background_type,
        color: &mut self.background_color,
        picture: &mut self.background_image_path,
        colors: &mut self.mesh_colors,
        generator: &mut self.mesh_generator,
        points: &mut self.mesh_points,
        seed: &mut self.mesh_seed,
        warp_percent: &mut self.mesh_warp_percent,
      },
      background,
    );
    if let Background::Mesh { locked_colors, .. } = background {
      locked_colors.clone_into(&mut self.mesh_locked_colors);
    }
  }

  /// Whether `other` draws the same background as this canvas, read from
  /// the fields its kind draws.
  pub fn wears_background_of(&self, other: &Self) -> bool {
    self.background_type == other.background_type
      && match self.background_type.as_str() {
        "image" => self.background_image_path == other.background_image_path,
        "mesh" => {
          self.mesh_colors == other.mesh_colors
            && self.mesh_generator == other.mesh_generator
            && self.mesh_points == other.mesh_points
            && self.mesh_seed == other.mesh_seed
            && self.mesh_warp_percent == other.mesh_warp_percent
        }
        _ => self.background_color == other.background_color,
      }
  }
}

#[cfg(test)]
mod tests;
