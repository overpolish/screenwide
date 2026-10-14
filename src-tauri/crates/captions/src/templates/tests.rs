// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use super::{
  bundled, bundled_templates, caption_templates, load_user_templates, parse_manifest,
  CaptionTemplateId, CaptionTemplateSource, FORMAT_VERSION,
};
use crate::style::CaptionStyle;

const USER: CaptionTemplateSource = CaptionTemplateSource::User;

fn templates_folder() -> std::path::PathBuf {
  std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

#[test]
fn every_template_folder_is_bundled_and_listed_once_in_order() {
  let mut on_disk: Vec<String> = std::fs::read_dir(templates_folder())
    .unwrap()
    .filter_map(|entry| {
      let entry = entry.unwrap();
      entry
        .path()
        .join("template.json")
        .is_file()
        .then(|| entry.file_name().to_string_lossy().into_owned())
    })
    .collect();
  on_disk.sort();
  let index = bundled::index().unwrap();
  let mut listed = index.templates.clone();
  listed.sort();
  assert_eq!(listed, on_disk);
  assert_eq!(bundled::MANIFESTS.len(), on_disk.len());
  let ids: Vec<&str> = bundled_templates().iter().map(|t| t.id.as_str()).collect();
  assert_eq!(ids, index.templates);
  assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
  assert!(ids.contains(&index.default.as_str()));
}

#[test]
fn every_bundled_template_is_complete_and_names_its_own_template() {
  for template in bundled_templates() {
    assert!(template.style.is_valid(), "{}", template.id);
    assert_eq!(template.style.template, template.id);
    assert_eq!(template.source, CaptionTemplateSource::Bundled);
    // Each is a whole example of a package, to be shared as one.
    assert!(template.info.author.is_some(), "{}", template.id);
    assert!(template.info.version.is_some(), "{}", template.id);
    assert!(template.info.description.is_some(), "{}", template.id);
    // It brings every family its style sets, and the app carries each file
    // and licence it declares.
    for family in [&template.style.font, &template.style.accent_font]
      .into_iter()
      .filter_map(|font| font.family.as_deref())
    {
      assert!(
        template.fonts.iter().any(|font| font.family == family),
        "{} sets {family} without bringing it",
        template.id
      );
    }
    for file in super::fonts::files_of(&template.fonts) {
      assert!(
        bundled::font_file(file).is_some(),
        "{} declares {file}",
        template.id
      );
    }
  }
  // And every file the app carries for them is declared by one.
  for (file, _) in bundled::FONT_FILES {
    assert!(
      bundled_templates()
        .iter()
        .any(|template| super::fonts::files_of(&template.fonts).any(|each| each == file)),
      "{file} is carried for no template"
    );
  }
}

#[test]
fn base_json_spells_out_every_option_of_the_default_style() {
  let base: serde_json::Value =
    serde_json::from_str(&std::fs::read_to_string(templates_folder().join("base.json")).unwrap())
      .unwrap();
  assert_eq!(base["formatVersion"], FORMAT_VERSION);
  let mut default = serde_json::to_value(CaptionStyle::default()).unwrap();
  default.as_object_mut().unwrap().remove("template");
  let keys = |value: &serde_json::Value| {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
  };
  assert_eq!(keys(&base["style"]), keys(&default));
  let read: CaptionStyle = serde_json::from_value(base["style"].clone()).unwrap();
  assert_eq!(read, CaptionStyle::default());
}

#[test]
fn an_old_id_reads_as_the_template_it_became_and_an_unknown_one_is_kept() {
  let read = |id: &str| serde_json::from_value::<CaptionTemplateId>(id.into()).unwrap();
  for old in ["reveal", "subtitle", "duet"] {
    assert_eq!(read(old).as_str(), "clean");
  }
  let style: CaptionStyle = serde_json::from_str(r#"{"template":"duet"}"#).unwrap();
  assert_eq!(style.template.as_str(), "clean");
  let unknown = read("someone-elses");
  assert_eq!(unknown.as_str(), "someone-elses");
  // A template the app does not have still draws: as the default one.
  assert_eq!(
    unknown.shipped_style(),
    CaptionTemplateId::default().shipped_style()
  );
}

#[test]
fn a_manifest_is_turned_away_for_what_a_style_cannot_hold() {
  let manifest = |rest: &str| {
    format!(r#"{{"formatVersion":{FORMAT_VERSION},"id":"mine","name":"Mine",{rest}}}"#)
  };
  assert!(parse_manifest(&manifest(r#""style":{"maxWords":4}"#), USER).is_ok());
  let exit = r#""style":{"motion":{"captionOut":[{"property":"opacity","keyframes":[{"atMs":0,"value":1},{"atMs":200,"value":0}]}]}}"#;
  assert!(parse_manifest(&manifest(exit), USER).is_ok());
  let long = "x".repeat(41);
  for wrong in [
    format!(
      r#"{{"formatVersion":{},"id":"mine","name":"Mine","style":{{}}}}"#,
      FORMAT_VERSION + 1
    ),
    r#"{"id":"mine","style":{}}"#.to_owned(),
    manifest(r#""style":{"maxWord":4}"#),
    manifest(r#""style":{"template":"box"}"#),
    manifest(r#""style":{"maxWords":400}"#),
    manifest(r#""style":{},"effects":[]"#),
    r#"{"formatVersion":2,"id":"Not An Id","style":{}}"#.to_owned(),
    r#"{"formatVersion":1,"id":"mine","style":{"motion":"pop"}}"#.to_owned(),
    // Format 3 names its template, briefly.
    format!(r#"{{"formatVersion":{FORMAT_VERSION},"id":"mine","style":{{}}}}"#),
    manifest(r#""author":" ","style":{}"#).replace(r#""name":"Mine","#, ""),
    format!(r#"{{"formatVersion":{FORMAT_VERSION},"id":"mine","name":"{long}","style":{{}}}}"#),
    // Exits and rolling words came with format 4.
    manifest(exit).replace(&format!(":{FORMAT_VERSION},"), ":3,"),
    manifest(r#""style":{"visibleWords":3}"#).replace(&format!(":{FORMAT_VERSION},"), ":3,"),
    manifest(r#""style":{"visibleWords":9}"#),
  ] {
    assert!(parse_manifest(&wrong, USER).is_err(), "{wrong}");
  }
}

#[test]
fn reads_an_older_manifest_named_by_its_id_and_its_entrance_as_its_motion() {
  let old = parse_manifest(
    r#"{"formatVersion":1,"id":"mine","style":{"entrance":"pop"}}"#,
    USER,
  )
  .unwrap();
  assert_eq!(old.info.name, "mine");
  assert_eq!(
    old.style.motion,
    CaptionTemplateId::from("pop").shipped_style().motion
  );
  let error = parse_manifest(
    &format!(
      r#"{{"formatVersion":{},"id":"mine","style":{{}}}}"#,
      FORMAT_VERSION + 1
    ),
    USER,
  )
  .unwrap_err();
  assert!(error.to_string().contains("newer version"), "{error}");
}

#[test]
fn reads_your_templates_from_a_folder_and_leaves_out_what_cannot_be_read() {
  let _reading = super::user::TESTS_READING_FOLDERS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let folder =
    std::env::temp_dir().join(format!("screenwide-user-templates-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&folder);
  let write = |name: &str, json: &str| {
    std::fs::create_dir_all(folder.join(name)).unwrap();
    std::fs::write(folder.join(name).join("template.json"), json).unwrap();
  };
  let manifest = |id: &str, name: &str| {
    format!(
      r#"{{"formatVersion":{FORMAT_VERSION},"id":"{id}","name":"{name}","style":{{"maxWords":3}}}}"#
    )
  };
  write("a", &manifest("zesty", "Zesty"));
  write("b", &manifest("bold-mine", "Bold"));
  write("c", &manifest("box", "Not the app's"));
  write("d", &manifest("zesty", "Again"));
  write("e", "{ not json");
  std::fs::create_dir_all(folder.join("f")).unwrap();
  let problems = load_user_templates(&folder);
  let mine: Vec<_> = caption_templates()
    .into_iter()
    .filter(|template| template.source == USER)
    .map(|template| (template.id.to_string(), template.info.name))
    .collect();
  // By name, after the app's own; a taken id, a broken manifest and a
  // folder without one each left out and said why.
  assert_eq!(
    mine,
    [
      ("bold-mine".to_owned(), "Bold".to_owned()),
      ("zesty".to_owned(), "Zesty".to_owned())
    ]
  );
  assert_eq!(problems.len(), 4, "{problems:?}");
  assert_eq!(
    CaptionTemplateId::from("zesty").shipped_style().max_words,
    3
  );
  // Read again without them, they are gone.
  std::fs::remove_dir_all(&folder).unwrap();
  assert!(load_user_templates(&folder).is_empty());
  assert!(caption_templates()
    .iter()
    .all(|template| template.source != USER));
}
