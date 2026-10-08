// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Embeds every translation in `locales/` into the binary, so adding a
//! language is adding its folder: no list to update here or in `i18n.rs`.
//! The webviews read the same files through Vite.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub fn embed(out_dir: &Path) {
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
  let root = Path::new(&manifest_dir).join("../locales");
  // A directory is scanned for any change below it, so a new language or
  // file reruns this step as well as an edit does.
  println!("cargo:rerun-if-changed={}", root.display());

  let mut locales: Vec<_> = fs::read_dir(&root)
    .expect("locales/ exists beside src-tauri/")
    .map(|entry| entry.expect("locales/ is readable").path())
    .filter(|path| path.is_dir())
    .collect();
  locales.sort();

  let mut source = String::from("pub(crate) static LOCALES: &[(&str, &[&str])] = &[\n");
  for locale in locales {
    let name = locale
      .file_name()
      .and_then(|name| name.to_str())
      .expect("UTF-8 locale name");
    let mut files: Vec<_> = fs::read_dir(&locale)
      .expect("locale folder is readable")
      .map(|entry| entry.expect("locale folder is readable").path())
      .filter(|path| path.extension().is_some_and(|extension| extension == "ftl"))
      .collect();
    files.sort();
    write!(source, "  ({name:?}, &[").unwrap();
    for file in files {
      let file = file.canonicalize().expect("translation file resolves");
      write!(
        source,
        "include_str!({:?}), ",
        file.to_str().expect("UTF-8 path")
      )
      .unwrap();
    }
    source.push_str("]),\n");
  }
  source.push_str("];\n");
  fs::write(out_dir.join("locales.rs"), source).expect("OUT_DIR is writable");
}
