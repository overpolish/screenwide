// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shaders the build compiled draw what the same shaders compiled from
//! WGSL draw. The build lays out each shader's bindings itself, the way it
//! expects wgpu to lay out the shader's layout; a wrong expectation, after a
//! wgpu upgrade say, shows here as a different picture rather than as a
//! broken first frame in the editor.

use screenwide_imaging::CapturedImage;
use std::sync::Arc;

use super::canvas_pipelines::CanvasPipelines;
use super::render_test_helpers::{compositor, gpu, target, view, OUTPUT};
use super::*;
use crate::precompiled_shaders::{EVERY_KIND, NO_ANNOTATIONS};
use screenwide_annotations::arrow::new_arrow;
use screenwide_annotations::counter::new_counter;
use screenwide_annotations::redact::new_redact;
use screenwide_annotations::{Annotation, AnnotationPoint, AnnotationShape};
use screenwide_output::fixtures::test_output_settings;

/// A 64 pixel square of colour changing across it, so a misread texture or
/// sampler shows.
fn picture() -> CapturedImage {
  let rgba = (0..64u32)
    .flat_map(|y| (0..64u32).map(move |x| (x, y)))
    .flat_map(|(x, y)| [(x * 4) as u8, (y * 4) as u8, ((x + y) * 2) as u8, 255])
    .collect();
  CapturedImage {
    width: 64,
    height: 64,
    rgba,
  }
}

/// An arrow, a counter, a redaction and a focus: annotation buffers, the
/// counter atlas and its sampler, the tile binning, the redaction pre-pass,
/// and the focus chain and its gather all read.
fn annotations() -> Vec<Annotation> {
  let point = |x, y| AnnotationPoint { x, y };
  let mut redaction = new_redact("redaction".to_owned(), point(40.0, 8.0), None);
  if let AnnotationShape::Redact { end, .. } = &mut redaction.shape {
    *end = point(60.0, 28.0);
  }
  vec![
    new_arrow("arrow".to_owned(), point(8.0, 8.0), point(56.0, 40.0), None),
    new_counter("counter".to_owned(), point(20.0, 48.0), 1, None, None),
    redaction,
    screenwide_annotations::focus::model::new_focus(
      "focus".to_owned(),
      [point(4.0, 30.0), point(30.0, 60.0)],
      None,
    ),
  ]
}

/// The whole canvas, BGRA: the picture with a rounded shadowed frame on a
/// mesh background, under `annotations`, drawn as an export draws it: with
/// the variant with no annotation or the one with every kind.
fn drawn(compositor: &Compositor, annotations: Vec<Annotation>) -> Vec<u8> {
  let _every = compositor.hold_every_annotation_kind();
  let image = picture();
  let mut settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  settings.background_type = "mesh".to_owned();
  settings.mesh_generator = "aurora".to_owned();
  settings.mesh_seed = 50_628;
  settings.mesh_colors = vec!["#8FE3FF".into(), "#FF9BE3".into(), "#C9B8FF".into()];
  settings.mesh_points.clear();
  settings.background_radius_percent = 15.9;
  settings.drop_shadow = true;
  (
    settings.image_width,
    settings.crop_width,
    settings.crop_height,
  ) = (64.0, 64.0, 64.0);
  (
    settings.image_x,
    settings.image_y,
    settings.crop_x,
    settings.crop_y,
  ) = (400.0, 100.0, 400.0, 100.0);
  settings.annotations = annotations;
  let source = compositor.screenshot_source(&image).unwrap();
  let prepared = crate::annotation_gpu::prepared_arrows(
    &settings.annotations,
    (image.width, image.height),
    &settings,
    Some(&image),
    None,
    None,
  )
  .unwrap();
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &source,
      &settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
    )
    .unwrap();
  gpu().read_texture(&output).unwrap()
}

/// A compositor whose shaders are all compiled from WGSL: its own passes,
/// and every canvas variant, which compositors on a device share.
fn compiled_from_wgsl() -> Compositor {
  let mut compositor = screenwide_gpu::compiling_wgsl(compositor);
  compositor.canvas = Arc::new(CanvasPipelines::compiled_from_wgsl(&gpu().device));
  compositor
}

/// The WGSL build compiles for the device's newest shader model or Metal
/// language and the build for an older one, and the two may round a channel
/// apart by one. A binding laid out wrongly reads the wrong texture or buffer
/// and differs by far more.
fn assert_same_pixels(precompiled: &[u8], compiled: &[u8], what: &str) {
  let differences: Vec<u8> = precompiled
    .iter()
    .zip(compiled)
    .map(|(a, b)| a.abs_diff(*b))
    .filter(|difference| *difference > 1)
    .collect();
  assert!(
    differences.is_empty(),
    "{what}: {} of {} bytes differ from the WGSL build by more than one, at most {}",
    differences.len(),
    compiled.len(),
    differences.iter().max().unwrap_or(&0)
  );
}

#[test]
#[ignore = "requires a GPU adapter"]
fn the_precompiled_shaders_draw_what_wgsl_draws() {
  assert!(
    gpu()
      .device
      .features()
      .contains(wgpu::Features::PASSTHROUGH_SHADERS),
    "this device takes no compiled shaders"
  );
  assert!(
    NO_ANNOTATIONS.is_some() && EVERY_KIND.is_some(),
    "the build compiled no canvas variants for this platform"
  );
  let precompiled = compositor();
  let compiled = compiled_from_wgsl();
  let bare = drawn(&precompiled, Vec::new());
  assert_same_pixels(&bare, &drawn(&compiled, Vec::new()), "no annotation");
  let annotated = drawn(&precompiled, annotations());
  assert_ne!(annotated, bare, "the annotations drew nothing");
  assert_same_pixels(&annotated, &drawn(&compiled, annotations()), "every kind");
}
