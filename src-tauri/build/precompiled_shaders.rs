// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The app crate's shaders compiled ahead of time
//! (`screenwide_wgsl::precompile`) into `OUT_DIR/precompiled_shaders.rs`,
//! which `src/lib.rs` includes.

use screenwide_wgsl::{assembled, Precompiled};

#[path = "../src/annotate/overlay/overlay_layout.rs"]
mod overlay_layout;
#[path = "../src/editor/preview_platform/audio_ribbon/ribbon_layout.rs"]
mod ribbon_layout;

pub(crate) fn precompile() {
  println!("cargo:rerun-if-changed=src/annotate/overlay/overlay_layout.rs");
  println!("cargo:rerun-if-changed=src/editor/preview_platform/audio_ribbon/ribbon_layout.rs");
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
        "ANNOTATE_OVERLAY",
        assembled("annotate_overlay"),
        &overlay_layout::ANNOTATE_OVERLAY,
      ),
      drawing(
        "AUDIO_RIBBON",
        assembled("audio_ribbon"),
        &ribbon_layout::AUDIO_RIBBON,
      ),
    ],
  );
}
