// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The whole picture laid over the canvas: on a phone, the strips the phone
//! cuts off or the bars it adds, then the app and the system's own bars laid
//! out against the phone's screen; without one, the app against the canvas.

use std::fmt::Write;

use super::phone::{screen_on, Phone, Rect, SystemBars};
use super::svg::{Anchor, Svg, DEFS, WIDTH};
use super::{reels, shorts, tiktok, OverlayRequest, PlatformOverlay, Words};

/// The SVG for `request` over a canvas `canvas` pixels, and the rect it
/// covers, in the canvas's pixels: the canvas.
pub(crate) fn document(request: OverlayRequest, canvas: (f64, f64)) -> (String, Rect) {
  let canvas_rect = Rect {
    x: 0.0,
    y: 0.0,
    width: canvas.0,
    height: canvas.1,
  };
  let (screen, insets) = match request.phone {
    Some(view) => (screen_on(canvas, view), view.phone.insets()),
    None => (canvas_rect, (0.0, 0.0)),
  };
  let bounds = canvas_rect.union(screen);
  let radius = request
    .phone
    .map_or(0.0, |view| view.phone.corner() * screen.width);
  let screen_path = rounded(screen, radius);
  let canvas_path = rounded(canvas_rect, 0.0);
  let mut body = String::new();
  // What the phone cuts off the canvas: dimmed and hatched.
  if request.phone.is_some() {
    let _ = write!(
      body,
      "<path d=\"{canvas_path}{screen_path}\" fill-rule=\"evenodd\" fill=\"#000\" fill-opacity=\"0.6\"/>\
<path d=\"{canvas_path}{screen_path}\" fill-rule=\"evenodd\" fill=\"url(#cut)\"/>"
    );
  }
  let words = Words::new();
  let mut svg = Svg::new(screen.width / screen.height.max(1.0), insets);
  match request.platform {
    Some(PlatformOverlay::Tiktok) => tiktok::draw(&mut svg, &words),
    Some(PlatformOverlay::YoutubeShorts) => shorts::draw(&mut svg, &words),
    Some(PlatformOverlay::InstagramReels) => reels::draw(&mut svg, &words),
    None => {}
  }
  if let Some(view) = request.phone {
    system_bars(&mut svg, view.phone);
  }
  let scale = screen.width / WIDTH;
  let _ = write!(
    body,
    "<g clip-path=\"url(#screen)\"><g transform=\"translate({:.2} {:.2}) scale({scale:.5})\">{}</g></g>",
    screen.x,
    screen.y,
    svg.into_body()
  );
  if request.phone.is_some() {
    let _ = write!(
      body,
      "<path d=\"{screen_path}\" fill=\"none\" stroke=\"#ffffff\" stroke-opacity=\"0.55\" stroke-width=\"1.5\"/>"
    );
  }
  let document = format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" viewBox=\"{x:.2} {y:.2} {w:.2} {h:.2}\">\
<defs>{DEFS}<clipPath id=\"screen\"><path d=\"{screen_path}\"/></clipPath></defs>{body}</svg>",
    x = bounds.x,
    y = bounds.y,
    w = bounds.width,
    h = bounds.height,
  );
  (document, bounds)
}

/// `rect` as a path, its corners rounded `radius`.
fn rounded(rect: Rect, radius: f64) -> String {
  let r = radius.min(rect.width / 2.0).min(rect.height / 2.0).max(0.0);
  let (x, y, right, bottom) = (rect.x, rect.y, rect.x + rect.width, rect.y + rect.height);
  format!(
    "M{:.2} {y:.2}H{:.2}A{r:.2} {r:.2} 0 0 1 {right:.2} {:.2}V{:.2}A{r:.2} {r:.2} 0 0 1 {:.2} {bottom:.2}H{:.2}A{r:.2} {r:.2} 0 0 1 {x:.2} {:.2}V{:.2}A{r:.2} {r:.2} 0 0 1 {:.2} {y:.2}Z",
    x + r,
    right - r,
    y + r,
    bottom - r,
    right - r,
    x + r,
    bottom - r,
    y + r,
    x + r,
  )
}

/// The system's own parts over the app, in the screen's units: the clock
/// and the camera cut-out at the top, the home indicator at the foot.
fn system_bars(svg: &mut Svg, phone: Phone) {
  let (top, bottom) = phone.insets();
  let top = top * svg.height;
  let bottom = bottom * svg.height;
  let white = "#ffffff";
  match phone.bars() {
    SystemBars::Island => {
      svg.rect(
        (WIDTH * 0.345, top * 0.18, WIDTH * 0.31, top * 0.6),
        top * 0.3,
        "#000000",
      );
      svg.text(
        "9:41",
        (WIDTH * 0.15, top * 0.62),
        top * 0.34,
        600,
        white,
        Anchor::Middle,
        false,
      );
    }
    SystemBars::HomeButton => {
      svg.text(
        "9:41",
        (WIDTH / 2.0, top * 0.75),
        top * 0.6,
        600,
        white,
        Anchor::Middle,
        false,
      );
    }
    SystemBars::PunchHole => {
      svg.circle((WIDTH / 2.0, top * 0.5), top * 0.22, "#000000");
      svg.text(
        "9:41",
        (WIDTH * 0.08, top * 0.66),
        top * 0.42,
        500,
        white,
        Anchor::Start,
        false,
      );
    }
  }
  if bottom > 0.0 {
    let (width, y) = match phone.bars() {
      SystemBars::PunchHole => (WIDTH * 0.25, svg.foot() - bottom * 0.5),
      _ => (WIDTH * 0.36, svg.foot() - bottom * 0.4),
    };
    let height = (bottom * 0.14).max(6.0);
    svg.rect(
      ((WIDTH - width) / 2.0, y - height / 2.0, width, height),
      height / 2.0,
      white,
    );
  }
}
