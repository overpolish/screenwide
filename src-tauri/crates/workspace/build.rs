// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "src/shader_layouts.rs"]
mod shader_layouts;

use screenwide_wgsl::{assembled, Precompiled};

fn main() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  screenwide_wgsl::assemble(&[
    (
      "workspace_backdrop",
      &["src/shaders/workspace_backdrop.wgsl"],
    ),
    (
      "workspace_magnifier",
      &[
        // First: it carries the module's directive.
        "src/shaders/workspace_magnifier.wgsl",
        "../osc/src/gpu/shaders/lens.wgsl",
      ],
    ),
  ]);
  let drawing = |name, source, layout| Precompiled {
    name,
    source,
    layout,
    entry_points: &["vs_main", "fs_main"],
  };
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[
      drawing(
        "BACKDROP",
        assembled("workspace_backdrop"),
        &shader_layouts::BACKDROP,
      ),
      drawing(
        "LOUPE",
        assembled("workspace_magnifier"),
        &shader_layouts::LOUPE,
      ),
    ],
  );
}
