// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::TemplatePackage;
use crate::templates::{caption_templates, user, CaptionTemplateId, CaptionTemplateSource};

fn scratch(name: &str) -> PathBuf {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-template-package-{name}-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  folder
}

fn manifest(id: &str, name: &str) -> String {
  format!(r#"{{"formatVersion":3,"id":"{id}","name":"{name}","style":{{"maxWords":3}}}}"#)
}

/// A zip at `path` holding `entries`, each a name and its contents.
fn zipped(path: &Path, entries: &[(&str, &[u8])]) {
  let mut zip = ZipWriter::new(std::fs::File::create(path).unwrap());
  for (name, bytes) in entries {
    zip.start_file(*name, SimpleFileOptions::default()).unwrap();
    zip.write_all(bytes).unwrap();
  }
  zip.finish().unwrap();
}

fn paths(package: &TemplatePackage) -> Vec<&str> {
  package
    .files
    .iter()
    .map(|(path, _)| path.as_str())
    .collect()
}

#[test]
fn carries_its_manifest_and_fonts_and_leaves_the_rest_behind() {
  let folder = scratch("folder");
  std::fs::write(folder.join("template.json"), manifest("mine", "Mine")).unwrap();
  std::fs::write(folder.join("README.md"), "notes").unwrap();
  std::fs::create_dir_all(folder.join("fonts/nested")).unwrap();
  std::fs::write(folder.join("fonts/Face.TTF"), b"font").unwrap();
  std::fs::write(folder.join("fonts/Face-OFL.txt"), b"licence").unwrap();
  std::fs::write(folder.join("fonts/run.sh"), b"#!/bin/sh").unwrap();
  let package = TemplatePackage::read_folder(&folder).unwrap();
  assert_eq!(
    paths(&package),
    ["template.json", "fonts/Face-OFL.txt", "fonts/Face.TTF"]
  );
  assert_eq!(package.template.source, CaptionTemplateSource::User);
  // Written as a zip and read back, it is the same package.
  let zip = folder.join("mine.zip");
  package.write_zip(&zip).unwrap();
  let read = TemplatePackage::read_zip(&zip).unwrap();
  assert_eq!(read.files, package.files);
  assert_eq!(read.template, package.template);
}

#[test]
fn reads_a_zips_manifest_at_the_top_or_in_one_folder_there() {
  let folder = scratch("layouts");
  let at_top = folder.join("top.zip");
  zipped(
    &at_top,
    &[("template.json", manifest("top", "Top").as_bytes())],
  );
  assert_eq!(
    TemplatePackage::read_zip(&at_top)
      .unwrap()
      .template
      .id
      .as_str(),
    "top"
  );
  let in_folder = folder.join("folder.zip");
  zipped(
    &in_folder,
    &[
      ("inner/template.json", manifest("inner", "Inner").as_bytes()),
      ("inner/fonts/Face.otf", b"font"),
      ("__MACOSX/inner/._template.json", b"resource fork"),
    ],
  );
  let package = TemplatePackage::read_zip(&in_folder).unwrap();
  assert_eq!(paths(&package), ["template.json", "fonts/Face.otf"]);
}

#[test]
fn turns_away_a_zip_it_cannot_take_whole() {
  let folder = scratch("refused");
  let refused = |entries: &[(&str, &[u8])]| {
    let path = folder.join("refused.zip");
    zipped(&path, entries);
    TemplatePackage::read_zip(&path).unwrap_err()
  };
  let mine = manifest("mine", "Mine");
  assert!(refused(&[("../template.json", mine.as_bytes())]).contains("outside"));
  assert!(refused(&[("fonts/Face.ttf", b"font")]).contains("no template.json"));
  assert!(refused(&[
    ("a/template.json", mine.as_bytes()),
    ("b/template.json", mine.as_bytes()),
  ])
  .contains("more than one"));
  // The app's own ids are its own.
  assert!(refused(&[("template.json", manifest("box", "Box").as_bytes())]).contains("app's own"));
  assert!(!refused(&[("template.json", b"{ not json")]).is_empty());
  // A font it declares but does not carry would be set in another face.
  let declaring = r#"{"formatVersion":3,"id":"mine","name":"Mine","fonts":[{"file":"fonts/Face.ttf","family":"Face","weights":[400]}],"style":{}}"#;
  assert!(refused(&[("template.json", declaring.as_bytes())]).contains("fonts/Face.ttf"));
  let carried = folder.join("carried.zip");
  zipped(
    &carried,
    &[
      ("template.json", declaring.as_bytes()),
      ("fonts/Face.ttf", b"font"),
    ],
  );
  assert!(TemplatePackage::read_zip(&carried).is_ok());
  assert!(TemplatePackage::read_zip(&folder.join("absent.zip")).is_err());
}

#[test]
fn installs_in_place_of_one_with_its_id_and_exports_the_apps_own() {
  let _reading = user::TESTS_READING_FOLDERS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let source = scratch("source");
  let templates = scratch("installed");
  // One of yours read from a folder named otherwise gives way too.
  std::fs::create_dir_all(templates.join("old-name")).unwrap();
  std::fs::write(
    templates.join("old-name/template.json"),
    manifest("mine", "Old"),
  )
  .unwrap();
  crate::load_user_templates(&templates);
  std::fs::write(source.join("template.json"), manifest("mine", "New")).unwrap();
  TemplatePackage::read_folder(&source)
    .unwrap()
    .install(&templates)
    .unwrap();
  let mine: Vec<_> = caption_templates()
    .into_iter()
    .filter(|template| template.source == CaptionTemplateSource::User)
    .map(|template| template.info.name)
    .collect();
  assert_eq!(mine, ["New"]);
  assert!(!templates.join("old-name").exists());
  assert!(templates.join("mine/template.json").is_file());
  // The app's own exports as its manifest, as bundled, with the fonts it
  // brings and their licences.
  let pop = TemplatePackage::of(&CaptionTemplateId::from("pop")).unwrap();
  assert_eq!(pop.template.source, CaptionTemplateSource::Bundled);
  assert_eq!(
    paths(&pop),
    [
      "template.json",
      "fonts/InstrumentSerif-Regular.ttf",
      "fonts/InstrumentSerif-OFL-1.1.txt",
      "fonts/InstrumentSerif-Italic.ttf"
    ]
  );
  assert!(TemplatePackage::of(&CaptionTemplateId::from("nobodys")).is_err());
  crate::load_user_templates(&scratch("empty"));
}
