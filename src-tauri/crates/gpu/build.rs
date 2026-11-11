// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "src/shader_layouts.rs"]
mod shader_layouts;

use screenwide_wgsl::{assembled, Precompiled};

fn main() {
  println!("cargo:rerun-if-changed=src/shader_layouts.rs");
  screenwide_wgsl::assemble(&[("mips", &["src/shaders/mips.wgsl"])]);
  screenwide_wgsl::precompile(
    "precompiled_shaders.rs",
    &[Precompiled {
      name: "MIPS",
      source: assembled("mips"),
      layout: &shader_layouts::MIPS,
      entry_points: &["vs_main", "fs_main"],
    }],
  );
}
