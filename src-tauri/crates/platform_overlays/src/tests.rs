// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// How opaque the picture is at `point`, a share of its width and height.
fn alpha_at(picture: &OverlayPicture, (x, y): (f64, f64)) -> u8 {
  let column = (x * f64::from(picture.size.0)) as usize;
  let row = (y * f64::from(picture.size.1)) as usize;
  picture.pixels[(row * picture.size.0 as usize + column) * 4 + 3]
}

fn app(platform: PlatformOverlay) -> OverlayRequest {
  OverlayRequest {
    platform: Some(platform),
    phone: None,
  }
}

#[test]
fn every_overlay_draws_its_tab_bar_and_leaves_the_middle_clear() {
  for platform in [
    PlatformOverlay::Tiktok,
    PlatformOverlay::YoutubeShorts,
    PlatformOverlay::InstagramReels,
  ] {
    let picture = draw(app(platform), (540, 960)).expect("an overlay");
    assert_eq!(picture.pixels.len(), 540 * 960 * 4);
    assert_eq!(picture.origin, (0.0, 0.0));
    // The tab bar is opaque across the bottom; the picture's middle, which
    // the video is seen through, is left clear.
    assert_eq!(alpha_at(&picture, (0.05, 0.99)), 255, "{platform:?}");
    assert_eq!(alpha_at(&picture, (0.4, 0.4)), 0, "{platform:?}");
  }
}

#[test]
fn a_taller_frame_keeps_the_tab_bar_on_the_bottom_edge() {
  let picture = draw(app(PlatformOverlay::Tiktok), (500, 1083)).expect("an overlay");
  assert_eq!(alpha_at(&picture, (0.05, 0.995)), 255);
  assert_eq!(alpha_at(&picture, (0.05, 0.6)), 0);
}

#[test]
fn a_tall_phone_cuts_the_sides_of_the_video() {
  let filled = screen_on(
    (1_080.0, 1_920.0),
    PhoneView {
      phone: Phone::Iphone17Pro,
    },
  );
  // 1920 tall at 1206:2622 is 883 wide: about 98 pixels cut each side.
  assert!((filled.width - 883.1).abs() < 0.1, "{filled:?}");
  assert!((filled.x - 98.4).abs() < 0.1, "{filled:?}");
  assert_eq!(filled.height, 1_920.0);
}

#[test]
fn a_project_saved_with_bars_reads_as_filled() {
  let view: PhoneView =
    serde_json::from_str(r#"{"phone":"pixel-10","fit":"fit"}"#).expect("a phone");
  assert_eq!(view.phone, Phone::Pixel10);
}
