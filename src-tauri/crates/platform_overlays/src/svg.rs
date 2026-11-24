// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An overlay written as SVG on a screen 1080 units wide, as tall as the
//! screen's shape makes it, so the apps' measures, taken on a 1080 by 1920
//! frame, are written as they were taken. What sits at the top is placed
//! below the system's status bar and what sits at the bottom above its home
//! indicator, each from its own edge, so a taller screen keeps each part on
//! its edge.

use std::fmt::Write;

use super::icons::Glyph;

/// The overlay's width in units.
pub(crate) const WIDTH: f64 = 1080.0;

/// The status bar's room the apps' measures, taken on 1080 by 1920 frames,
/// already leave at the top, in units.
const TEMPLATE_STATUS: f64 = 60.0;

const FONT: &str = "Helvetica Neue, Segoe UI, Arial, sans-serif";

pub(crate) struct Svg {
  body: String,
  /// The screen's height in units.
  pub(crate) height: f64,
  /// The room the system keeps at the top and the foot, in units.
  top: f64,
  bottom: f64,
}

/// How text sits on its point.
#[derive(Clone, Copy)]
pub(crate) enum Anchor {
  Start,
  Middle,
}

impl Svg {
  /// A screen `aspect` wide to its height, the system keeping `insets` of
  /// its height at the top and the foot.
  pub(crate) fn new(aspect: f64, insets: (f64, f64)) -> Self {
    let height = WIDTH / aspect.max(0.1);
    Self {
      body: String::new(),
      height,
      top: insets.0 * height,
      bottom: insets.1 * height,
    }
  }

  /// `y` units above the bottom edge of the room the app has.
  pub(crate) fn up(&self, y: f64) -> f64 {
    self.height - self.bottom - y
  }

  /// `y` units below the top of the frame the apps' measures were taken
  /// on. Those frames already leave the 60 units a status bar takes, so on
  /// a phone only the system's room past them moves the part down.
  pub(crate) fn down(&self, y: f64) -> f64 {
    (self.top - TEMPLATE_STATUS).max(0.0) + y
  }

  /// `glyph` `size` units square with its top-left corner at `x`, `y`, in
  /// `color`, its drop shadow under it where `shadow`.
  pub(crate) fn glyph(
    &mut self,
    glyph: &Glyph,
    (x, y): (f64, f64),
    size: f64,
    color: &str,
    shadow: bool,
  ) {
    let filter = if shadow {
      " filter=\"url(#shadow)\""
    } else {
      ""
    };
    let _ = write!(
      self.body,
      "<g transform=\"translate({x:.1} {y:.1}) scale({:.4})\"{filter}>",
      size / 24.0
    );
    self.parts(glyph, color);
    self.body.push_str("</g>");
  }

  fn parts(&mut self, glyph: &Glyph, color: &str) {
    match glyph {
      Glyph::Fill(path) => {
        let _ = write!(
          self.body,
          "<path d=\"{path}\" fill=\"{color}\" fill-rule=\"evenodd\"/>"
        );
      }
      Glyph::Stroke(path) => {
        let _ = write!(
          self.body,
          "<path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>"
        );
      }
      Glyph::Parts(parts) => {
        for part in *parts {
          self.parts(part, color);
        }
      }
    }
  }

  /// A glyph centred on `centre`.
  pub(crate) fn glyph_at(
    &mut self,
    glyph: &Glyph,
    centre: (f64, f64),
    size: f64,
    color: &str,
    shadow: bool,
  ) {
    self.glyph(
      glyph,
      (centre.0 - size / 2.0, centre.1 - size / 2.0),
      size,
      color,
      shadow,
    );
  }

  /// One line of `text` with its baseline at `y`.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn text(
    &mut self,
    text: &str,
    (x, y): (f64, f64),
    size: f64,
    weight: u16,
    color: &str,
    anchor: Anchor,
    shadow: bool,
  ) {
    let anchor = match anchor {
      Anchor::Start => "start",
      Anchor::Middle => "middle",
    };
    let filter = if shadow {
      " filter=\"url(#shadow)\""
    } else {
      ""
    };
    let _ = write!(
      self.body,
      "<text x=\"{x:.1}\" y=\"{y:.1}\" font-family=\"{FONT}\" font-size=\"{size:.1}\" font-weight=\"{weight}\" fill=\"{color}\" text-anchor=\"{anchor}\"{filter}>{}</text>",
      escaped(text)
    );
  }

  pub(crate) fn circle(&mut self, (x, y): (f64, f64), radius: f64, fill: &str) {
    let _ = write!(
      self.body,
      "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"{radius:.1}\" fill=\"{fill}\"/>"
    );
  }

  pub(crate) fn ring(&mut self, (x, y): (f64, f64), radius: f64, stroke: &str, width: f64) {
    let _ = write!(
      self.body,
      "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"{radius:.1}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{width:.1}\"/>"
    );
  }

  pub(crate) fn rect(
    &mut self,
    (x, y, width, height): (f64, f64, f64, f64),
    radius: f64,
    fill: &str,
  ) {
    let _ = write!(
      self.body,
      "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"{radius:.1}\" fill=\"{fill}\"/>"
    );
  }

  pub(crate) fn outline(
    &mut self,
    (x, y, width, height): (f64, f64, f64, f64),
    radius: f64,
    stroke: &str,
    line: f64,
  ) {
    let _ = write!(
      self.body,
      "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"{radius:.1}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{line:.1}\"/>"
    );
  }

  /// A shade darkening the picture from `from` units above the bottom down
  /// to `to`, as the apps lay one under their captions.
  pub(crate) fn bottom_shade(&mut self, from: f64, to: f64) {
    let (top, bottom) = (self.up(from), self.up(to));
    let _ = write!(
      self.body,
      "<rect x=\"0\" y=\"{top:.1}\" width=\"{WIDTH}\" height=\"{:.1}\" fill=\"url(#shade)\"/>",
      bottom - top
    );
  }

  /// A person's picture: a grey disc with a silhouette.
  pub(crate) fn avatar(&mut self, centre: (f64, f64), radius: f64) {
    self.circle(centre, radius, "#c9ccd1");
    self.glyph_at(
      &super::icons::PROFILE_SOLID,
      (centre.0, centre.1 + radius * 0.12),
      radius * 1.6,
      "#8a8f98",
      false,
    );
  }

  /// What has been drawn, in the screen's units.
  pub(crate) fn into_body(self) -> String {
    self.body
  }
}

/// What the drawings refer to: the glyphs' drop shadow and the shade the
/// apps lay under their captions.
pub(crate) const DEFS: &str = "<filter id=\"shadow\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\"><feDropShadow dx=\"0\" dy=\"1.5\" stdDeviation=\"2.5\" flood-color=\"#000\" flood-opacity=\"0.35\"/></filter>\
<linearGradient id=\"shade\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#000\" stop-opacity=\"0\"/><stop offset=\"1\" stop-color=\"#000\" stop-opacity=\"0.45\"/></linearGradient>\
<pattern id=\"cut\" width=\"10\" height=\"10\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\"><line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"10\" stroke=\"#ffffff\" stroke-opacity=\"0.35\" stroke-width=\"3\"/></pattern>";

impl Svg {
  /// The bottom edge of the screen, for parts that reach under the
  /// system's home indicator.
  pub(crate) fn foot(&self) -> f64 {
    self.height
  }
}

fn escaped(text: &str) -> String {
  text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
}
