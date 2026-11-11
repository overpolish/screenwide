// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every shader this crate draws with, compiled ahead of time
//! (`screenwide_wgsl::precompile`) into `OUT_DIR/precompiled_shaders.rs`,
//! which `src/precompiled_shaders.rs` includes. Of the canvas shader, the
//! variant with no annotation and the one with every kind.

use screenwide_wgsl::{assembled, Precompiled};

#[path = "../src/shader_layouts.rs"]
mod shader_layouts;

/// The switch `compositor/canvas_variants.rs` turns off for the variant with
/// no annotation. The variant with every kind is the shader as written.
const DRAWN: &str = "const annotations_drawn: bool = true;";
const LEFT_OUT: &str = "const annotations_drawn: bool = false;";

const DRAWING: &[&str] = &["vs_main", "fs_main"];

pub(crate) fn precompile() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  let preview = assembled("preview");
  assert!(
    preview.contains(DRAWN),
    "preview.wgsl no longer declares `{DRAWN}`, which the variant with no annotation turns off"
  );
  let drawing = |name, source, layout| Precompiled {
    name,
    source,
    layout,
    entry_points: DRAWING,
  };
  let mut shaders = vec![
    drawing(
      "NO_ANNOTATIONS",
      preview.replacen(DRAWN, LEFT_OUT, 1),
      &shader_layouts::PREVIEW,
    ),
    drawing("EVERY_KIND", preview.clone(), &shader_layouts::PREVIEW),
    Precompiled {
      name: "TILES",
      source: preview,
      layout: &shader_layouts::TILES,
      entry_points: &["bin_annotations"],
    },
    drawing(
      "REDACT_CELLS",
      assembled("redact_cells"),
      &shader_layouts::REDACT,
    ),
    drawing(
      "REDACT_ROWS",
      assembled("redact_rows"),
      &shader_layouts::REDACT,
    ),
    drawing(
      "REDACT_PAINT",
      assembled("redact_paint"),
      &shader_layouts::REDACT,
    ),
    Precompiled {
      name: "FOCUS_COPY",
      source: assembled("focus_copy"),
      layout: &shader_layouts::FOCUS_LEVEL,
      entry_points: &["vs_main", "fs_downsample"],
    },
    drawing(
      "FOCUS_BLUR",
      assembled("focus_blur"),
      &shader_layouts::FOCUS_GATHER,
    ),
    drawing(
      "MARK_BLUR",
      assembled("annotation_blur"),
      &shader_layouts::MARK_BLUR,
    ),
  ];
  // Only the macOS export splits frames into planes on the GPU.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    shaders.push(Precompiled {
      name: "VIDEO_PLANES",
      source: assembled("video_planes"),
      layout: &shader_layouts::VIDEO_PLANES,
      entry_points: &["vs_main", "fs_from_planes", "fs_luma", "fs_chroma"],
    });
  }
  screenwide_wgsl::precompile("precompiled_shaders.rs", &shaders);
}
