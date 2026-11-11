// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "src/shader_layouts.rs"]
mod shader_layouts;

use screenwide_wgsl::{assembled, Precompiled};

fn main() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  screenwide_wgsl::assemble(&[("edge_fill", &["src/scene/edge_fill.wgsl"])]);
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[Precompiled {
      name: "EDGE_FILL",
      source: assembled("edge_fill"),
      layout: &shader_layouts::EDGE_FILL,
      entry_points: &["vertex_main", "fragment_main"],
    }],
  );
}
