// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! For build scripts. WGSL has no includes, so a shader made of several files
//! is listed as the parts it joins, in order. Each module is validated with
//! naga and written to `OUT_DIR/<name>.wgsl`, which the renderer embeds with
//! `include_str!`: a broken shader fails the build instead of the first draw.

use std::path::Path;

/// Joins and validates each of `modules`, a name and its parts relative to
/// the crate being built, into `OUT_DIR`.
pub fn assemble(modules: &[(&str, &[&str])]) {
  let output = std::env::var_os("OUT_DIR").expect("Cargo supplied OUT_DIR");
  for (name, parts) in modules {
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
