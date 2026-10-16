// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a template draws beyond its type and its background, each element
//! placed by the control points it names. An element is one of a fixed set
//! the app knows how to draw, so a template stays data however it is shared.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The widest a tail's base and tip are, in ems.
const MAX_TAIL_EMS: f64 = 4.0;

/// One element a template draws.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TextElement {
  /// A tapered tail from the text's box to control point `to`: a round base
  /// with a radius of `base` ems hidden inside the box, narrowing to a
  /// rounded tip with a radius of `tip` ems, blended into the box over
  /// `blend` ems so it leaves the edge in a curve. It leaves the edge its
  /// point reaches furthest past and slides along it as the point moves, and
  /// is not drawn while the point is tucked into the box. It draws out of
  /// the box once the box is in, and back into it before the box leaves, so
  /// a long tail never sweeps in with the box around it. Only a text with a
  /// box draws one.
  Tail {
    to: String,
    #[serde(default = "tail_base")]
    base: f64,
    #[serde(default = "tail_tip")]
    tip: f64,
    #[serde(default = "tail_blend")]
    blend: f64,
  },
}

/// A tail's shape as the text box draws it, in ems.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TailShape {
  pub base: f32,
  pub tip: f32,
  pub blend: f32,
}

impl Default for TailShape {
  /// The tail the app's own Label template draws.
  fn default() -> Self {
    Self {
      base: tail_base() as f32,
      tip: tail_tip() as f32,
      blend: tail_blend() as f32,
    }
  }
}

fn tail_base() -> f64 {
  0.5
}

fn tail_tip() -> f64 {
  0.04
}

fn tail_blend() -> f64 {
  0.3
}

impl TextElement {
  /// The control point the element is drawn to.
  pub fn control(&self) -> &str {
    match self {
      Self::Tail { to, .. } => to,
    }
  }

  /// The tail's shape, where the element is one.
  pub fn tail(&self) -> Option<TailShape> {
    match *self {
      Self::Tail {
        base, tip, blend, ..
      } => Some(TailShape {
        base: base as f32,
        tip: tip as f32,
        blend: blend as f32,
      }),
    }
  }

  pub(crate) fn normalized(&self) -> Self {
    let ems = |value: f64, fallback: f64| {
      if value.is_finite() {
        value.clamp(0.0, MAX_TAIL_EMS)
      } else {
        fallback
      }
    };
    match self {
      Self::Tail {
        to,
        base,
        tip,
        blend,
      } => {
        let base = ems(*base, tail_base());
        Self::Tail {
          to: to.clone(),
          base,
          tip: ems(*tip, tail_tip()).min(base),
          blend: ems(*blend, tail_blend()),
        }
      }
    }
  }
}
