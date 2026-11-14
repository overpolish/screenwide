// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a canvas sits on, kept whole: a saved preset carries one, and so does
//! a scene that gives the recording a background of its own. The output
//! canvas spreads the same values across its flat fields.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The name settings written before generators existed carry, and the one an
/// absent field falls back to.
pub const DEFAULT_GENERATOR: &str = "mesh";

pub fn default_generator() -> String {
  DEFAULT_GENERATOR.to_owned()
}

/// One blob of a mesh gradient, in shares of the canvas.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BackgroundMeshPoint {
  pub radius_x: f64,
  pub radius_y: f64,
  pub rotation: f64,
  pub x: f64,
  pub y: f64,
}

/// What sits behind the picture, as a saved preset or a scene carries it.
///
/// The three kinds are the three ways a canvas is filled: one colour, a mesh
/// of them, or a picture of your own. The editor spreads the same values
/// across the flat fields of an output canvas; a preset keeps them together,
/// since a preset is one background rather than part of one.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Background {
  #[serde(rename_all = "camelCase")]
  Solid { color: String },
  #[serde(rename_all = "camelCase")]
  Image { path: String },
  #[serde(rename_all = "camelCase")]
  Mesh {
    colors: Vec<String>,
    seed: u32,
    warp_percent: f64,
    /// Which picture this paints. Presets saved before the ported generators
    /// existed carry none, and are the app's own blob mesh.
    #[serde(default = "default_generator")]
    generator: String,
    #[serde(default)]
    locked_colors: Vec<bool>,
    /// Present only for the classic mesh generator, which is the only one
    /// that reads blobs; the others draw from their colours and seed alone.
    #[serde(default)]
    #[ts(as = "Option<Vec<BackgroundMeshPoint>>", optional)]
    points: Vec<BackgroundMeshPoint>,
  },
}

#[cfg(test)]
mod tests;
