// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The DirectWrite and Direct2D objects annotation type is set and drawn
//! with: one set for the process, behind a lock, and never released.
//!
//! Not one set per thread: a thread-local is released as its thread exits,
//! under the loader lock, and releasing DirectWrite there deadlocks the exit.

mod draw;
mod faces;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, PoisonError};

use windows::{
  core::{w, IUnknown, HSTRING, PCWSTR},
  Win32::Graphics::{
    Direct2D::{
      Common::{D2D1_ALPHA_MODE_IGNORE, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
      D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory, ID2D1SolidColorBrush,
      D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES,
      D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_RENDER_TARGET_USAGE_NONE,
    },
    DirectWrite::{
      DWriteCreateFactory, IDWriteFactory6, IDWriteFontCollection2, IDWriteFontSetBuilder1,
      IDWriteRenderingParams3, IDWriteTextFormat3, IDWriteTextLayout, IDWriteTypography,
      DWRITE_FACTORY_TYPE_ISOLATED, DWRITE_FONT_AXIS_TAG_ITALIC, DWRITE_FONT_AXIS_TAG_WEIGHT,
      DWRITE_FONT_AXIS_VALUE, DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC, DWRITE_FONT_FEATURE,
      DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES, DWRITE_FONT_LINE_GAP_USAGE_DEFAULT,
      DWRITE_FONT_METRICS, DWRITE_FONT_METRICS1, DWRITE_FONT_STRETCH_NORMAL,
      DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT,
      DWRITE_GRID_FIT_MODE_DISABLED, DWRITE_LINE_SPACING, DWRITE_LINE_SPACING_METHOD_UNIFORM,
      DWRITE_PIXEL_GEOMETRY_FLAT, DWRITE_RENDERING_MODE1_DEFAULT, DWRITE_TEXT_METRICS,
      DWRITE_TEXT_RANGE, DWRITE_WORD_WRAPPING_NO_WRAP,
    },
    Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    Gdi::{CreateCompatibleDC, HDC},
  },
};

use super::INTER;
use crate::text::bundled::BUNDLED_FILES;
use crate::text::face::TypeFace;

/// The size lines are measured at before scaling to the size asked for.
/// DirectWrite's widths scale evenly with size, so one format serves every
/// size without creating another per measurement.
pub(super) const REFERENCE_SIZE: f64 = 256.0;

/// One face as the engine sets it: its format at [`REFERENCE_SIZE`], and
/// its ascent and descent in ems.
#[derive(Clone)]
pub(super) struct Face {
  pub(super) reference: IDWriteTextFormat3,
  pub(super) ascent: f64,
  pub(super) descent: f64,
  /// Where the family is found: the app's own collection, or the system's.
  system: bool,
  family: HSTRING,
  weight: f32,
  italic: bool,
}

pub(super) struct Engine {
  factory: IDWriteFactory6,
  /// The app's own families: Inter, and those its caption templates use.
  collection: IDWriteFontCollection2,
  /// Every family installed on the system, for captions.
  system: IDWriteFontCollection2,
  tabular: IDWriteTypography,
  params: IDWriteRenderingParams3,
  dc: HDC,
  target: ID2D1DCRenderTarget,
  ink: ID2D1SolidColorBrush,
  /// Inter's ascent and descent, in ems.
  inter_metrics: (f64, f64),
  /// Every face asked for so far.
  faces: RefCell<HashMap<TypeFace, Face>>,
}

// Made on one thread and used from others. DirectWrite's objects are
// free-threaded, and the Direct2D ones and the context are only reached
// through the lock.
unsafe impl Send for Engine {}

static ENGINE: LazyLock<Result<Mutex<Engine>, String>> = LazyLock::new(|| {
  unsafe { Engine::new() }
    .map(Mutex::new)
    .map_err(|error| format!("Windows could not set up annotation type: {error}"))
});

/// Runs `work` with the engine, set up on first use.
pub(super) fn with<R>(work: impl FnOnce(&Engine) -> Result<R, String>) -> Result<R, String> {
  let engine = ENGINE.as_ref().map_err(Clone::clone)?;
  work(&engine.lock().unwrap_or_else(PoisonError::into_inner))
}

/// How far a laid-out line advances, trailing spaces included, as GDI's
/// extent and Core Text's typographic width both count them.
pub(super) fn width(layout: &IDWriteTextLayout) -> f64 {
  let mut metrics = DWRITE_TEXT_METRICS::default();
  match unsafe { layout.GetMetrics(&mut metrics) } {
    Ok(()) => f64::from(metrics.widthIncludingTrailingWhitespace),
    Err(_) => 0.0,
  }
}

impl Engine {
  unsafe fn new() -> windows::core::Result<Self> {
    unsafe {
      let factory: IDWriteFactory6 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_ISOLATED)?;
      let loader = factory.CreateInMemoryFontFileLoader()?;
      factory.RegisterFontFileLoader(&loader)?;
      let builder = factory.CreateFontSetBuilder()?;
      // Inter goes first, so it is the set's first face, whose metrics are
      // read below. No owner, so the loader keeps its own copy of each.
      for bytes in std::iter::once(INTER).chain(BUNDLED_FILES.iter().copied()) {
        let file = loader.CreateInMemoryFontFileReference(
          &factory,
          bytes.as_ptr().cast(),
          bytes.len() as u32,
          None::<&IUnknown>,
        )?;
        IDWriteFontSetBuilder1::AddFontFile(&builder, &file)?;
      }
      let set = builder.CreateFontSet()?;
      let collection =
        factory.CreateFontCollectionFromFontSet(&set, DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC)?;
      let mut metrics = DWRITE_FONT_METRICS1::default();
      set
        .GetFontFaceReference(0)?
        .CreateFontFace()?
        .GetMetrics(&mut metrics);
      let em = f64::from(metrics.Base.designUnitsPerEm.max(1));
      let (ascent, descent) = (
        f64::from(metrics.Base.ascent) / em,
        f64::from(metrics.Base.descent) / em,
      );
      let system = factory.GetSystemFontCollection(false, DWRITE_FONT_FAMILY_MODEL_TYPOGRAPHIC)?;
      let tabular = factory.CreateTypography()?;
      tabular.AddFontFeature(DWRITE_FONT_FEATURE {
        nameTag: DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES,
        parameter: 1,
      })?;
      // The coverage becomes the shader's alpha, so it stays linear: no
      // gamma, no contrast lift, and outlines left where the layout put them
      // rather than fitted to the pixel grid.
      let params = factory.CreateCustomRenderingParams(
        1.0,
        0.0,
        0.0,
        0.0,
        DWRITE_PIXEL_GEOMETRY_FLAT,
        DWRITE_RENDERING_MODE1_DEFAULT,
        DWRITE_GRID_FIT_MODE_DISABLED,
      )?;
      let d2d: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
      let target = d2d.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
        pixelFormat: D2D1_PIXEL_FORMAT {
          format: DXGI_FORMAT_B8G8R8A8_UNORM,
          alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        // One device-independent pixel to the atlas pixel.
        dpiX: 96.0,
        dpiY: 96.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
      })?;
      let ink = target.CreateSolidColorBrush(
        &D2D1_COLOR_F {
          r: 1.0,
          g: 1.0,
          b: 1.0,
          a: 1.0,
        },
        None,
      )?;
      let dc = CreateCompatibleDC(None);
      if dc.is_invalid() {
        return Err(windows::core::Error::from_thread());
      }
      Ok(Self {
        factory,
        collection,
        system,
        tabular,
        params,
        dc,
        target,
        ink,
        inter_metrics: (ascent, descent),
        faces: RefCell::new(HashMap::new()),
      })
    }
  }

  /// One line set in `format`, with tabular figures when `tabular`.
  pub(super) fn layout(
    &self,
    format: &IDWriteTextFormat3,
    text: &[u16],
    tabular: bool,
  ) -> Result<IDWriteTextLayout, String> {
    let layout = || -> windows::core::Result<IDWriteTextLayout> {
      unsafe {
        // Lines never wrap, so the layout box is only an origin.
        let layout = self.factory.CreateTextLayout(text, format, 0.0, 0.0)?;
        if tabular {
          layout.SetTypography(
            &self.tabular,
            DWRITE_TEXT_RANGE {
              startPosition: 0,
              length: text.len() as u32,
            },
          )?;
        }
        Ok(layout)
      }
    };
    layout().map_err(|error| format!("Windows could not lay out annotation type: {error}"))
  }
}
