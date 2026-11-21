// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's own vector drawings of its cursors, drawn at
//! [`VECTOR_SCALE`] times their point size.
//!
//! `NSCursor` hands out bitmaps, and only the arrow and the I-beam come at
//! 10x; the rest stop at 2x, which a large pointer on a retina recording
//! stretches five times and more into a blur. macOS keeps most of its
//! cursors as PDFs beside an `info.plist` holding the hotspot and the shadow
//! the window server adds under them. That folder is not public API, so a
//! cursor whose PDF is missing or unreadable keeps its bitmap.

use std::path::{Path, PathBuf};

use image::RgbaImage;
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_app_kit::{
  NSBitmapImageRep, NSColor, NSColorSpace, NSCompositingOperation, NSDeviceRGBColorSpace,
  NSGraphicsContext, NSImage, NSImageRep, NSShadow,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};
use screenwide_records::cursor::CursorStyle;

/// Texels per point the vector cursors are drawn at: crisp at the largest
/// pointer size (5x) on a retina recording (2x), and under some zoom.
pub(super) const VECTOR_SCALE: f64 = 16.0;

const CURSORS: &str = "/System/Library/Frameworks/ApplicationServices.framework/Versions/A/\
                       Frameworks/HIServices.framework/Versions/A/Resources/cursors";

/// The folder a style's drawing is in. The arrow and the I-beam have none,
/// and ship large enough as bitmaps.
fn folder(style: CursorStyle) -> Option<&'static str> {
  Some(match style {
    CursorStyle::ClosedHand => "closedhand",
    CursorStyle::ContextMenu => "contextualmenu",
    CursorStyle::Crosshair => "cross",
    CursorStyle::DisappearingItem => "poof",
    CursorStyle::DragCopy => "copy",
    CursorStyle::DragLink => "makealias",
    CursorStyle::NotAllowed => "notallowed",
    CursorStyle::OpenHand => "openhand",
    CursorStyle::PointingHand => "pointinghand",
    CursorStyle::ResizeHorizontal => "resizeleftright",
    CursorStyle::ResizeVertical => "resizeupdown",
    CursorStyle::VerticalIBeam => "ibeamvertical",
    CursorStyle::ZoomIn => "zoomin",
    CursorStyle::ZoomOut => "zoomout",
    _ => return None,
  })
}

/// Where `name`'s drawing is: a release's own redrawing first, in a folder
/// named after it (`macos27/pointinghand`), the newest release's where
/// several have one, else the shared one.
fn drawing(root: &Path, name: &str) -> Option<PathBuf> {
  let mut releases: Vec<PathBuf> = std::fs::read_dir(root)
    .ok()?
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("macos"))
    })
    .collect();
  releases.sort();
  releases
    .into_iter()
    .rev()
    .map(|release| release.join(name))
    .chain(std::iter::once(root.join(name)))
    .find(|folder| folder.join("cursor.pdf").is_file())
}

/// What `info.plist` says about a drawing: its hotspot in points from the
/// top left, and the shadow drawn under it.
#[derive(Debug, PartialEq)]
struct Info {
  frames: i64,
  hotspot: (f64, f64),
  shadow_blur: f64,
  shadow_color: [f64; 4],
  shadow_offset: (f64, f64),
}

fn info(folder: &Path) -> Option<Info> {
  let value = plist::Value::from_file(folder.join("info.plist")).ok()?;
  let dict = value.as_dictionary()?;
  let number = |key: &str| {
    dict.get(key).and_then(|value| {
      value
        .as_real()
        .or_else(|| value.as_signed_integer().map(|n| n as f64))
    })
  };
  let color = dict
    .get("shadowcolor")
    .and_then(plist::Value::as_array)
    .map(|parts| {
      let mut rgba = [0.0, 0.0, 0.0, 0.45];
      for (slot, part) in rgba.iter_mut().zip(parts) {
        if let Some(value) = part
          .as_real()
          .or_else(|| part.as_signed_integer().map(|n| n as f64))
        {
          *slot = value;
        }
      }
      rgba
    })
    .unwrap_or([0.0, 0.0, 0.0, 0.45]);
  Some(Info {
    frames: dict
      .get("frames")
      .and_then(plist::Value::as_signed_integer)
      .unwrap_or(1),
    hotspot: (number("hotx")?, number("hoty")?),
    shadow_blur: number("blur").unwrap_or(2.0),
    shadow_color: color,
    shadow_offset: (
      number("shadowoffsetx").unwrap_or(0.0),
      number("shadowoffsety").unwrap_or(-1.0),
    ),
  })
}

/// `style`'s vector drawing at [`VECTOR_SCALE`] texels per point, over the
/// box of `points` the cursor is drawn in, with its shadow, and its hotspot
/// in points from the top left; `None` where macOS has no drawing for it or
/// it cannot be read, or it is animated.
pub(super) fn vector_artwork(
  style: CursorStyle,
  points: (f64, f64),
) -> Option<(RgbaImage, (f64, f64))> {
  let folder = drawing(Path::new(CURSORS), folder(style)?)?;
  let info = info(&folder)?;
  if info.frames != 1 || points.0 <= 0.0 || points.1 <= 0.0 {
    return None;
  }
  let image = draw(&folder.join("cursor.pdf"), points, &info)?;
  Some((image, info.hotspot))
}

fn draw(pdf: &Path, points: (f64, f64), info: &Info) -> Option<RgbaImage> {
  let width = (points.0 * VECTOR_SCALE).round();
  let height = (points.1 * VECTOR_SCALE).round();
  let drawing =
    NSImage::initWithContentsOfFile(NSImage::alloc(), &NSString::from_str(pdf.to_str()?))?;
  // SAFETY: a null plane pointer asks AppKit to allocate the pixels itself.
  let blank = unsafe {
    NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
      NSBitmapImageRep::alloc(),
      std::ptr::null_mut(),
      width as isize,
      height as isize,
      8,
      4,
      true,
      false,
      NSDeviceRGBColorSpace,
      0,
      0,
    )
  }?;
  // Drawn in sRGB, as the bitmaps it stands in for are and as the export
  // is, rather than the display's own colours.
  let canvas = blank.bitmapImageRepByRetaggingWithColorSpace(&NSColorSpace::sRGBColorSpace())?;
  let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&canvas)?;
  NSGraphicsContext::saveGraphicsState_class();
  NSGraphicsContext::setCurrentContext(Some(&context));
  // The shadow the window server adds, in points, so scaled with the drawing.
  let shadow = NSShadow::new();
  shadow.setShadowBlurRadius(info.shadow_blur * VECTOR_SCALE);
  shadow.setShadowOffset(NSSize::new(
    info.shadow_offset.0 * VECTOR_SCALE,
    info.shadow_offset.1 * VECTOR_SCALE,
  ));
  let [red, green, blue, alpha] = info.shadow_color;
  shadow.setShadowColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
    red, green, blue, alpha,
  )));
  shadow.set();
  drawing.drawInRect_fromRect_operation_fraction(
    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height)),
    NSRect::ZERO,
    NSCompositingOperation::SourceOver,
    1.0,
  );
  context.flushGraphics();
  NSGraphicsContext::restoreGraphicsState_class();
  decode(&canvas)
}

/// The drawn pixels, read the way the system bitmaps are, so both reach the
/// GPU alike.
fn decode(canvas: &Retained<NSBitmapImageRep>) -> Option<RgbaImage> {
  let single = NSArray::from_slice(&[&**canvas as &NSImageRep]);
  let data = NSBitmapImageRep::TIFFRepresentationOfImageRepsInArray(&single)?;
  let image = image::load_from_memory(&data.to_vec()).ok()?.into_rgba8();
  (image.width() > 0 && image.height() > 0).then_some(image)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// An empty folder of the test's own.
  fn scratch(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
      "screenwide-cursor-vector-{name}-{}",
      std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
  }

  #[test]
  fn reads_a_cursors_hotspot_and_shadow() {
    let folder = scratch("info");
    std::fs::write(
      folder.join("info.plist"),
      r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>blur</key><real>2</real>
<key>frames</key><integer>1</integer>
<key>hotx</key><integer>15</integer>
<key>hoty</key><real>12</real>
<key>shadowcolor</key><array><real>0</real><real>0</real><real>0</real><real>0.45</real></array>
<key>shadowoffsetx</key><real>0</real>
<key>shadowoffsety</key><real>-1</real>
</dict></plist>"#,
    )
    .unwrap();
    assert_eq!(
      info(&folder),
      Some(Info {
        frames: 1,
        hotspot: (15.0, 12.0),
        shadow_blur: 2.0,
        shadow_color: [0.0, 0.0, 0.0, 0.45],
        shadow_offset: (0.0, -1.0),
      })
    );
  }

  #[test]
  fn prefers_the_newest_releases_own_drawing() {
    let root = scratch("releases");
    for folder in [
      "pointinghand",
      "macos26/pointinghand",
      "macos27/pointinghand",
    ] {
      std::fs::create_dir_all(root.join(folder)).unwrap();
      std::fs::write(root.join(folder).join("cursor.pdf"), b"").unwrap();
    }
    std::fs::create_dir_all(root.join("cross")).unwrap();
    std::fs::write(root.join("cross/cursor.pdf"), b"").unwrap();
    assert_eq!(
      drawing(&root, "pointinghand"),
      Some(root.join("macos27/pointinghand"))
    );
    assert_eq!(drawing(&root, "cross"), Some(root.join("cross")));
  }

  /// The system's own crosshair, where this Mac has it: drawn at the full
  /// scale over its point box, with ink in it.
  #[test]
  fn draws_the_systems_crosshair_at_full_scale() {
    let Some((image, hotspot)) = vector_artwork(CursorStyle::Crosshair, (24.0, 24.0)) else {
      return;
    };
    assert_eq!((image.width(), image.height()), (384, 384));
    assert_eq!(hotspot, (11.0, 11.0));
    assert!(image.pixels().any(|pixel| pixel[3] == 255 && pixel[0] < 64));
    assert!(image
      .pixels()
      .any(|pixel| pixel[3] == 255 && pixel[0] > 192));
  }
}
