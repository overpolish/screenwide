// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's labels set by Core Text: Inter for chrome and Roboto Mono
//! for readouts, both with tabular figures, drawn at `SUPERSAMPLE` times the
//! output and box-filtered down, as the Windows raster is.

use std::sync::Once;

use objc2_foundation::{NSBundle, NSString};
use screenwide_compositor::platform::type_device::TypeDevice;
use screenwide_text::engine::face::TypeFace;

use super::super::region_osc::host::TextRaster;
use super::super::region_osc::text::SUPERSAMPLE;

unsafe extern "C" {
  fn CTFontManagerRegisterFontsForURL(
    url: *const std::ffi::c_void,
    scope: u32,
    error: *mut *mut std::ffi::c_void,
  ) -> bool;
}

/// Roboto Mono ships in the bundle's fonts; the process registers it once.
fn register_mono() {
  static REGISTERED: Once = Once::new();
  REGISTERED.call_once(|| {
    let bundle = NSBundle::mainBundle();
    let url = bundle.URLForResource_withExtension_subdirectory(
      Some(&NSString::from_str("RobotoMono-VariableFont_wght")),
      Some(&NSString::from_str("ttf")),
      Some(&NSString::from_str("fonts")),
    );
    if let Some(url) = url {
      // Safety: an NSURL is a CFURL; scope 1 is the process.
      unsafe {
        CTFontManagerRegisterFontsForURL(
          objc2::rc::Retained::as_ptr(&url).cast(),
          1,
          std::ptr::null_mut(),
        )
      };
    }
  });
}

pub(super) struct CoreTextRaster {
  device: TypeDevice,
}

impl CoreTextRaster {
  pub(super) fn new(font_size: f64, raster_scale: f64, mono: bool) -> Option<Self> {
    if font_size <= 0.0 || raster_scale <= 0.0 {
      return None;
    }
    let face = if mono {
      register_mono();
      TypeFace::new(Some("Roboto Mono"), 400, false)
    } else {
      TypeFace::new(None, 400, false)
    };
    TypeDevice::tabular_in_face(face, font_size * raster_scale)
      .ok()
      .map(|device| Self { device })
  }
}

impl TextRaster for CoreTextRaster {
  fn measure(&self, text: &str) -> Option<(i32, i32)> {
    let width = self.device.advance(text).ceil() as i32;
    let (ascent, descent) = self.device.vertical_metrics();
    let height = (ascent + descent).ceil() as i32;
    (width > 0 && height > 0).then_some((width, height))
  }

  fn coverage(&self, output: (i32, i32), runs: &[(String, i32, i32)]) -> Option<Vec<u8>> {
    let (output_width, output_height) = output;
    if output_width <= 0 || output_height <= 0 {
      return None;
    }
    let (width, height) = (output_width * SUPERSAMPLE, output_height * SUPERSAMPLE);
    let lines = runs
      .iter()
      .map(|(text, x, y)| ((f64::from(*x), f64::from(*y)), text.as_str()))
      .collect::<Vec<_>>();
    let samples = self
      .device
      .draw((width as u32, height as u32), &lines)
      .ok()?;
    let mut coverage = vec![0_u8; (output_width * output_height) as usize];
    let count = (SUPERSAMPLE * SUPERSAMPLE) as u32;
    for y in 0..output_height {
      for x in 0..output_width {
        let mut total = 0_u32;
        for sample_y in 0..SUPERSAMPLE {
          let row = ((y * SUPERSAMPLE + sample_y) * width + x * SUPERSAMPLE) as usize;
          total += samples[row..row + SUPERSAMPLE as usize]
            .iter()
            .map(|value| u32::from(*value))
            .sum::<u32>();
        }
        coverage[(y * output_width + x) as usize] = (total / count) as u8;
      }
    }
    Some(coverage)
  }
}
