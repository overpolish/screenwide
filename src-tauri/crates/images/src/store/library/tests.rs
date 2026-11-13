// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::OnceLock;

use image::{Rgba, RgbaImage};

use super::super::{import_bytes, import_file, import_found, initialize};
use super::*;

/// One store for every test in the process, as the app has one.
fn data() -> &'static Path {
  static DATA: OnceLock<PathBuf> = OnceLock::new();
  DATA.get_or_init(|| {
    let data = std::env::temp_dir().join(format!("screenwide-pictures-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    initialize(&data);
    data
  })
}

/// A PNG file of `colour`, half clear, so keeping it changes its bytes.
fn picture_file(name: &str, colour: [u8; 3]) -> PathBuf {
  let folder = data().join("scratch");
  std::fs::create_dir_all(&folder).unwrap();
  let path = folder.join(name);
  let [red, green, blue] = colour;
  RgbaImage::from_pixel(6, 3, Rgba([red, green, blue, 128]))
    .save(&path)
    .unwrap();
  path
}

fn listed(art: &ImageArt) -> Option<Picture> {
  pictures()
    .into_iter()
    .find(|picture| picture.art.asset == art.asset)
}

#[test]
fn a_picture_from_a_file_is_listed_by_the_file_name() {
  let art = import_file(&picture_file("Logo.png", [200, 10, 10])).unwrap();
  let picture = listed(&art).unwrap();
  assert_eq!(picture.title.as_deref(), Some("Logo"));
  assert_eq!(picture.art, art);
}

#[test]
fn a_pasted_picture_is_listed_without_a_name_until_a_file_names_it() {
  let file = picture_file("Badge.png", [10, 200, 10]);
  let art = import_bytes(&std::fs::read(&file).unwrap()).unwrap();
  assert_eq!(listed(&art).unwrap().title, None);
  assert_eq!(import_file(&file).unwrap(), art);
  assert_eq!(listed(&art).unwrap().title.as_deref(), Some("Badge"));
}

#[test]
fn the_store_s_own_file_brought_back_is_the_same_picture() {
  let art = import_file(&picture_file("Mark.png", [10, 10, 200])).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  let kept = stored_file(name).unwrap();
  assert_eq!(import_file(&kept).unwrap(), art);
}

#[test]
fn a_picture_kept_before_pictures_were_noted_is_described_from_its_file() {
  let art = import_file(&picture_file("Old.png", [90, 90, 10])).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  std::fs::remove_file(kept_path(directory().unwrap(), name)).unwrap();
  let picture = listed(&art).unwrap();
  assert_eq!(picture.title, None);
  assert!((picture.art.aspect - 2.0).abs() < 1e-9);
  assert!(picture.added_ms > 0);
}

#[test]
fn a_removed_picture_leaves_the_library() {
  let art = import_file(&picture_file("Gone.png", [10, 90, 90])).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  remove(name).unwrap();
  assert!(listed(&art).is_none());
  assert!(stored_file(name).is_none());
}

#[test]
fn a_removed_picture_a_project_hands_back_is_drawn_but_stays_out_until_added_again() {
  let file = picture_file("Carried.png", [200, 40, 40]);
  let art = import_file(&file).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  let project = data().join("project");
  std::fs::create_dir_all(&project).unwrap();
  let carried = project.join(stored_file(name).unwrap().file_name().unwrap());
  std::fs::copy(stored_file(name).unwrap(), &carried).unwrap();
  remove(name).unwrap();
  super::super::adopt(&carried).unwrap();
  assert!(stored_file(name).is_some());
  assert!(listed(&art).is_none());
  import_file(&file).unwrap();
  assert!(listed(&art).is_some());
}

#[test]
fn a_renamed_picture_is_listed_by_its_new_name() {
  let art =
    import_bytes(&std::fs::read(picture_file("Paste.png", [60, 20, 140])).unwrap()).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  assert!(rename(name, "").is_err());
  rename(name, "Badge").unwrap();
  assert_eq!(listed(&art).unwrap().title.as_deref(), Some("Badge"));
}

#[test]
fn a_found_picture_keeps_its_credit_and_its_title() {
  let bytes = std::fs::read(picture_file("found.png", [140, 60, 20])).unwrap();
  let credit = Credit {
    title: "Arrow".to_owned(),
    creator: Some("Ana".to_owned()),
    creator_url: None,
    license: "by".to_owned(),
    license_version: Some("4.0".to_owned()),
    license_url: None,
    source_url: Some("https://example.org/arrow".to_owned()),
    attribution: "\"Arrow\" by Ana is licensed under CC BY 4.0.".to_owned(),
  };
  let art = import_found(&bytes, "Arrow", credit.clone()).unwrap();
  let name = art.asset.strip_prefix(IMAGE_PREFIX).unwrap();
  assert_eq!(listed(&art).unwrap().title.as_deref(), Some("Arrow"));
  assert_eq!(super::super::credit_in(None, name), Some(credit));
}
