// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The app's shaders as the parts each joins, in order; `screenwide_wgsl`
//! joins and validates them into `OUT_DIR`.

const MODULES: &[(&str, &[&str])] = &[
  (
    "annotate_overlay",
    &[
      // First: it carries the module's directive.
      "src/annotate/shaders/annotate_overlay.wgsl",
      "crates/compositor/src/shaders/annotations.wgsl",
      "crates/compositor/src/shaders/annotation_counter.wgsl",
      "crates/compositor/src/shaders/annotation_text.wgsl",
      "crates/compositor/src/shaders/annotation_shape.wgsl",
      "crates/compositor/src/shaders/annotation_draw.wgsl",
      "crates/compositor/src/shaders/annotation_spotlight.wgsl",
      "crates/compositor/src/shaders/annotation_magnify.wgsl",
      "crates/compositor/src/shaders/annotation_highlight.wgsl",
    ],
  ),
  (
    "audio_ribbon",
    &["src/editor/preview_platform/shaders/audio_ribbon.wgsl"],
  ),
  (
    "workspace_magnifier",
    &[
      "src/editor/preview_platform/shaders/workspace_magnifier.wgsl",
      "crates/osc/src/gpu/shaders/lens.wgsl",
    ],
  ),
];

pub(crate) fn assemble() {
  screenwide_wgsl::assemble(MODULES);
}
