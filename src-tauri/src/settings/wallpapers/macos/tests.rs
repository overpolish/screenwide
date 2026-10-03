// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn content(provider: &str, configuration: &[u8], files: &[&str]) -> Value {
  let mut chosen = Dictionary::new();
  chosen.insert("Provider".to_owned(), Value::String(provider.to_owned()));
  chosen.insert(
    "Configuration".to_owned(),
    Value::Data(configuration.to_vec()),
  );
  let files = files
    .iter()
    .map(|address| {
      let mut file = Dictionary::new();
      file.insert("relative".to_owned(), Value::String((*address).to_owned()));
      Value::Dictionary(file)
    })
    .collect();
  chosen.insert("Files".to_owned(), Value::Array(files));
  let mut content = Dictionary::new();
  content.insert(
    "Choices".to_owned(),
    Value::Array(vec![Value::Dictionary(chosen)]),
  );
  Value::Dictionary(content)
}

fn entry(key: &str, provider: &str) -> Value {
  let mut inner = Dictionary::new();
  inner.insert("Content".to_owned(), content(provider, &[], &[]));
  let mut entry = Dictionary::new();
  entry.insert(key.to_owned(), Value::Dictionary(inner));
  Value::Dictionary(entry)
}

fn providers(store: &Value) -> Vec<String> {
  let mut found = Vec::new();
  desktop_choices(store, &mut found);
  found.into_iter().map(|one| one.provider).collect()
}

/// The shared desktop, a display's own, and a Space's own are all on
/// screen; the screen saver and a new display's default are not.
#[test]
fn reads_every_desktop_choice_and_nothing_else() {
  let mut separate = Dictionary::new();
  if let Value::Dictionary(desktop) = entry("Desktop", "display") {
    separate.extend(desktop);
  }
  if let Value::Dictionary(idle) = entry("Idle", "saver") {
    separate.extend(idle);
  }
  let mut displays = Dictionary::new();
  displays.insert("DISPLAY-UUID".to_owned(), Value::Dictionary(separate));
  let mut space = Dictionary::new();
  space.insert("Default".to_owned(), entry("Linked", "space"));
  let mut spaces = Dictionary::new();
  spaces.insert("SPACE-UUID".to_owned(), Value::Dictionary(space));
  let mut store = Dictionary::new();
  store.insert("AllSpacesAndDisplays".to_owned(), entry("Linked", "shared"));
  store.insert("Displays".to_owned(), Value::Dictionary(displays));
  store.insert("Spaces".to_owned(), Value::Dictionary(spaces));
  store.insert("SystemDefault".to_owned(), entry("Linked", "default"));
  assert_eq!(
    providers(&Value::Dictionary(store)),
    ["shared", "display", "space"]
  );
}

#[test]
fn a_choice_names_its_files_as_paths() {
  let Value::Dictionary(dictionary) = content(
    "com.apple.wallpaper.choice.image",
    &[],
    &["file:///Users/someone/My%20Picture.heic"],
  ) else {
    unreachable!()
  };
  let found = choice(&dictionary).unwrap();
  assert_eq!(
    found.files,
    [PathBuf::from("/Users/someone/My Picture.heic")]
  );
}

fn rendered(directory: &Path, folder: &str, provider: &str, configuration: &[u8]) -> PathBuf {
  let folder = directory.join(folder);
  std::fs::create_dir_all(&folder).unwrap();
  let mut metadata = Dictionary::new();
  metadata.insert("Content".to_owned(), content(provider, configuration, &[]));
  Value::Dictionary(metadata)
    .to_file_binary(folder.join(METADATA))
    .unwrap();
  let picture = folder.join(PICTURE);
  std::fs::write(&picture, b"picture").unwrap();
  picture
}

fn wanted(provider: &str, configuration: &[u8], files: Vec<PathBuf>) -> Choice {
  Choice {
    provider: provider.to_owned(),
    configuration: configuration.to_vec(),
    files,
  }
}

/// Two renders of one extension are told apart by their options, and a file
/// the person chose wins over any render.
#[test]
fn matches_a_choice_to_its_own_picture() {
  let directory = std::env::temp_dir().join("screenwide-active-wallpaper-test");
  let _ = std::fs::remove_dir_all(&directory);
  let dark = rendered(&directory, "A", "aerials", b"dark");
  let light = rendered(&directory, "B", "aerials", b"light");
  let mine = directory.join("mine.png");
  std::fs::write(&mine, b"picture").unwrap();
  let found = rendered_pictures(&directory);

  assert_eq!(
    picture_for(&wanted("aerials", b"dark", Vec::new()), &found),
    Some(dark)
  );
  assert_eq!(
    picture_for(&wanted("aerials", b"light", Vec::new()), &found),
    Some(light)
  );
  assert_eq!(
    picture_for(&wanted("aerials", b"dark", vec![mine.clone()]), &found),
    Some(mine)
  );
  assert_eq!(
    picture_for(&wanted("sequoia", b"", Vec::new()), &found),
    None
  );
  let _ = std::fs::remove_dir_all(&directory);
}
