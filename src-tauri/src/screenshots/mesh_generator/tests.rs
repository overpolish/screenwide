// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn numbers_are_the_wire_format() {
  for (index, generator) in every_generator().iter().enumerate() {
    assert_eq!(generator.id as usize, index);
    assert!((2..=MAXIMUM_GENERATOR_COLORS).contains(&generator.color_count));
  }
}

/// The picker's own table of generators, read off the TypeScript source:
/// each entry's `colorCount` followed by its `id`, in the order they are
/// written.
fn picker_generators() -> Vec<(String, usize)> {
  let source = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/components/shared/background-picker/background-generators.ts"
  ));
  let mut found = Vec::new();
  let mut count: Option<usize> = None;
  for line in source.lines() {
    let line = line.trim();
    if let Some(value) = line
      .strip_prefix("colorCount:")
      .and_then(|rest| rest.trim().trim_end_matches(',').parse::<usize>().ok())
    {
      count = Some(value);
    } else if let Some(rest) = line.strip_prefix("id: \"") {
      let Some(name) = rest.split('"').next() else {
        continue;
      };
      let Some(value) = count.take() else {
        panic!("{name} has no colorCount before it");
      };
      found.push((name.to_owned(), value));
    }
  }
  found
}

/// The shaders have their own parity tests; this one guards the two
/// generator tables, so adding a generator to one side alone fails loudly
/// rather than shipping a picker tile the backends cannot paint.
#[test]
fn the_picker_offers_the_generators_this_build_has() {
  let rust: Vec<(String, usize)> = every_generator()
    .iter()
    .map(|generator| (generator.name.to_owned(), generator.color_count))
    .collect();
  for (index, generator) in every_generator().iter().enumerate() {
    assert_eq!(generator.id as usize, index);
  }
  assert_eq!(picker_generators(), rust);
}

/// Every generator source, in the three shading languages. The ports are
/// meant to stay line for line, so a scan over one has to be a scan over
/// all three.
fn generator_sources() -> [(&'static str, &'static str); 6] {
  [
    (
      "mesh_generator_common.wgsl",
      include_str!("../mesh_generator_common.wgsl"),
    ),
    (
      "mesh_generators.wgsl",
      include_str!("../mesh_generators.wgsl"),
    ),
    (
      "mesh_generators_layered.wgsl",
      include_str!("../mesh_generators_layered.wgsl"),
    ),
    (
      "gpu_compositor_macos_generators.h",
      include_str!("../../editor/cursor_export/gpu_compositor_macos_generators.h"),
    ),
    (
      "gpu_compositor_macos_generators_layered.h",
      include_str!("../../editor/cursor_export/gpu_compositor_macos_generators_layered.h"),
    ),
    (
      "generators.hlsl",
      include_str!("../../editor/preview_platform/surface_windows/shaders/generators.hlsl"),
    ),
  ]
}

/// The three ports apply the speed at the same point, in a line that reads
/// the same but for the language's spelling of a local.
#[test]
fn every_port_scales_the_seconds_once_at_the_dispatch() {
  let mut dispatches = Vec::new();
  for (name, source) in generator_sources() {
    let applications = source.matches("time * speed").count();
    assert!(
      applications <= 1,
      "{name} applies the generator speed {applications} times"
    );
    if applications == 1 {
      dispatches.push(name);
    }
  }
  assert_eq!(
    dispatches,
    [
      "mesh_generators_layered.wgsl",
      "gpu_compositor_macos_generators_layered.h",
      "generators.hlsl",
    ],
    "the speed belongs at the three dispatches, one per port"
  );
}

#[test]
fn an_absent_name_is_the_original_mesh() {
  assert_eq!(mesh_generator("").map(|one| one.id), Some(0));
  assert_eq!(mesh_generator("mesh").map(|one| one.id), Some(0));
  assert_eq!(mesh_generator("silk").map(|one| one.id), Some(1));
  assert!(mesh_generator("nothing of the sort").is_none());
}
