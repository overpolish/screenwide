// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! WGSL has no includes, so a shader made of several files is listed here as
//! the parts it joins, in order. Each module is validated with naga and
//! written to `OUT_DIR/<name>.wgsl`, which the renderer embeds with
//! `include_str!`: a broken shader fails the build instead of the first draw.

use std::path::Path;

const MODULES: &[(&str, &[&str])] = &[
  (
    "annotate_overlay",
    &[
      // First: it carries the module's directive.
      "src/annotate/shaders/annotate_overlay.wgsl",
      "src/editor/preview_platform/shaders/annotations.wgsl",
      "src/editor/preview_platform/shaders/annotation_counter.wgsl",
      "src/editor/preview_platform/shaders/annotation_text.wgsl",
      "src/editor/preview_platform/shaders/annotation_shape.wgsl",
      "src/editor/preview_platform/shaders/annotation_draw.wgsl",
      "src/editor/preview_platform/shaders/annotation_spotlight.wgsl",
      "src/editor/preview_platform/shaders/annotation_magnify.wgsl",
      "src/editor/preview_platform/shaders/annotation_highlight.wgsl",
    ],
  ),
  (
    "annotation_blur",
    &["src/editor/preview_platform/shaders/annotation_blur.wgsl"],
  ),
  (
    "audio_ribbon",
    &["src/editor/preview_platform/shaders/audio_ribbon.wgsl"],
  ),
  (
    "camera_thumbnail",
    &["src/recording/shaders/camera_thumbnail.wgsl"],
  ),
  (
    "desktop_compositor",
    &["src/recording/shaders/desktop_compositor.wgsl"],
  ),
  (
    "mesh",
    &[
      "src/screenshots/mesh_generator_common.wgsl",
      "src/screenshots/mesh_generators.wgsl",
      "src/screenshots/mesh_generators_layered.wgsl",
      "src/screenshots/mesh.wgsl",
    ],
  ),
  (
    "osc",
    &[
      // First: it carries the module's directive.
      "src/osc/gpu/shaders/osc.wgsl",
      "src/osc/gpu/shaders/lens.wgsl",
    ],
  ),
  (
    "preview",
    &[
      // First: it carries the module's directive.
      "src/editor/preview_platform/shaders/preview.wgsl",
      "src/screenshots/mesh_generator_common.wgsl",
      "src/screenshots/mesh_generators.wgsl",
      "src/screenshots/mesh_generators_layered.wgsl",
      "src/editor/preview_platform/shaders/annotations.wgsl",
      "src/editor/preview_platform/shaders/annotation_counter.wgsl",
      "src/editor/preview_platform/shaders/annotation_text.wgsl",
      "src/editor/preview_platform/shaders/annotation_shape.wgsl",
      "src/editor/preview_platform/shaders/annotation_draw.wgsl",
      "src/editor/preview_platform/shaders/annotation_spotlight.wgsl",
      "src/editor/preview_platform/shaders/annotation_magnify.wgsl",
      "src/editor/preview_platform/shaders/annotation_highlight.wgsl",
      "src/editor/preview_platform/shaders/annotation_image.wgsl",
      "src/editor/preview_platform/shaders/annotation_layers.wgsl",
      "src/editor/preview_platform/shaders/annotation_tiles.wgsl",
    ],
  ),
  (
    "redact_cells",
    &[
      "src/editor/preview_platform/shaders/redact.wgsl",
      "src/editor/preview_platform/shaders/redact_cells.wgsl",
    ],
  ),
  (
    "redact_rows",
    &[
      "src/editor/preview_platform/shaders/redact.wgsl",
      "src/editor/preview_platform/shaders/redact_rows.wgsl",
    ],
  ),
  (
    "redact_paint",
    &[
      "src/editor/preview_platform/shaders/redact.wgsl",
      "src/editor/preview_platform/shaders/redact_paint.wgsl",
    ],
  ),
  (
    "video_planes",
    &["src/editor/preview_platform/shaders/video_planes.wgsl"],
  ),
  (
    "workspace_magnifier",
    &[
      "src/editor/preview_platform/shaders/workspace_magnifier.wgsl",
      "src/osc/gpu/shaders/lens.wgsl",
    ],
  ),
];

pub(crate) fn assemble() {
  let output = std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR");
  for (name, parts) in MODULES {
    let mut source = String::new();
    for part in *parts {
      println!("cargo:rerun-if-changed={part}");
      let text = std::fs::read_to_string(part)
        .unwrap_or_else(|error| panic!("read the shader {part}: {error}"));
      source.push_str(&text);
      source.push('\n');
    }
    validate(name, &source);
    std::fs::write(Path::new(&output).join(format!("{name}.wgsl")), source)
      .unwrap_or_else(|error| panic!("write the {name} shader: {error}"));
  }
}

fn validate(name: &str, source: &str) {
  let path = format!("{name}.wgsl");
  let module = naga::front::wgsl::parse_str(source)
    .unwrap_or_else(|error| panic!("{}", error.emit_to_string_with_path(source, &path)));
  naga::valid::Validator::new(
    naga::valid::ValidationFlags::all(),
    naga::valid::Capabilities::all(),
  )
  .validate(&module)
  .unwrap_or_else(|error| panic!("{}", error.emit_to_string_with_path(source, &path)));
}
