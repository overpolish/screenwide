// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas shader's variant with no annotation and its variant with every
//! kind, compiled ahead of time (`screenwide_wgsl::precompile`) into
//! `OUT_DIR/precompiled_shaders.rs`, which
//! `src/compositor/canvas_precompiled.rs` includes.

use std::path::Path;

use screenwide_wgsl::{Binding, Precompiled};

#[path = "../src/compositor/preview_bindings.rs"]
mod preview_bindings;

use preview_bindings::{PreviewBinding, PREVIEW_BINDINGS};

/// The switch `compositor/canvas_variants.rs` turns off for the variant with
/// no annotation. The variant with every kind is the shader as written.
const DRAWN: &str = "const annotations_drawn: bool = true;";
const LEFT_OUT: &str = "const annotations_drawn: bool = false;";

pub(crate) fn precompile(out_dir: &Path) {
  let shader = std::fs::read_to_string(out_dir.join("preview.wgsl"))
    .unwrap_or_else(|error| panic!("read the assembled preview shader: {error}"));
  assert!(
    shader.contains(DRAWN),
    "preview.wgsl no longer declares `{DRAWN}`, which the variant with no annotation turns off"
  );
  let bindings = PREVIEW_BINDINGS
    .iter()
    .zip(0..)
    .map(|(kind, binding)| {
      let kind = match kind {
        PreviewBinding::Uniform => Binding::Uniform,
        PreviewBinding::Storage => Binding::Storage,
        PreviewBinding::Texture => Binding::Texture,
        PreviewBinding::TextureArray => Binding::TextureArray,
        PreviewBinding::Sampler => Binding::Sampler,
      };
      (binding, kind)
    })
    .collect::<Vec<_>>();
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[
      Precompiled {
        name: "NO_ANNOTATIONS",
        source: shader.replacen(DRAWN, LEFT_OUT, 1),
        bindings: bindings.clone(),
      },
      Precompiled {
        name: "EVERY_KIND",
        source: shader,
        bindings,
      },
    ],
  );
}
