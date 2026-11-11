// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotate overlay's shader compiled ahead of time
//! (`screenwide_wgsl::precompile`) into `OUT_DIR/precompiled_shaders.rs`,
//! which `src/annotate/overlay/pipeline.rs` includes.

use screenwide_wgsl::{Binding, Precompiled};

#[path = "../src/annotate/overlay/overlay_bindings.rs"]
mod overlay_bindings;

use overlay_bindings::{OverlayBinding, OVERLAY_BINDINGS};

pub(crate) fn precompile() {
  println!("cargo:rerun-if-changed=src/annotate/overlay/overlay_bindings.rs");
  let out_dir = std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR");
  let source =
    std::fs::read_to_string(std::path::Path::new(&out_dir).join("annotate_overlay.wgsl"))
      .unwrap_or_else(|error| panic!("read the assembled annotate overlay shader: {error}"));
  let bindings = OVERLAY_BINDINGS
    .iter()
    .map(|&(binding, kind)| {
      let kind = match kind {
        OverlayBinding::Uniform => Binding::Uniform,
        OverlayBinding::Storage => Binding::Storage,
        OverlayBinding::Texture => Binding::Texture,
        OverlayBinding::Sampler => Binding::Sampler,
      };
      (binding, kind)
    })
    .collect();
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[Precompiled {
      name: "ANNOTATE_OVERLAY",
      source,
      bindings,
    }],
  );
}
