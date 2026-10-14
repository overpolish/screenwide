// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{pose, presets, CaptionMotion, CaptionProperty, Mirror, MAX_KEYFRAMES};
use crate::style::CaptionStyle;
use crate::templates::{parse_manifest, CaptionTemplateSource, FORMAT_VERSION};

fn motion(json: serde_json::Value) -> Result<CaptionMotion, String> {
  serde_json::from_value::<CaptionStyle>(serde_json::json!({ "motion": json }))
    .map(|style| style.motion)
    .map_err(|error| error.to_string())
}

/// How the app drew a popping caption before motion was data.
fn old_pop(elapsed_ms: f64) -> (f64, f64) {
  let progress = (elapsed_ms / 220.0).clamp(0.0, 1.0);
  let from_end = progress - 1.0;
  let eased = 1.0 + 3.5 * from_end.powi(3) + 2.5 * from_end.powi(2);
  (0.75 + 0.25 * eased, (elapsed_ms / 50.0).clamp(0.0, 1.0))
}

#[test]
fn the_presets_move_as_the_app_moved_captions_before() {
  let pop = presets::named("pop").unwrap();
  let fade = presets::named("fade").unwrap();
  for step in -10..100 {
    let elapsed = f64::from(step) * 5.0;
    let (scale, opacity) = old_pop(elapsed);
    for tracks in [&pop.caption, &pop.word] {
      let at = pose(tracks, elapsed, Mirror::default());
      assert!(
        (at.get(CaptionProperty::Scale) - scale).abs() < 1e-12,
        "{elapsed}"
      );
      assert!(
        (at.get(CaptionProperty::Opacity) - opacity).abs() < 1e-12,
        "{elapsed}"
      );
    }
    let faded = pose(&fade.word, elapsed, Mirror::default());
    assert_eq!(
      faded.get(CaptionProperty::Opacity),
      (elapsed / 90.0).clamp(0.0, 1.0)
    );
    assert_eq!(faded.get(CaptionProperty::Scale), 1.0);
    // A plain caption does not move as it comes on.
    assert_eq!(
      pose(&fade.caption, elapsed, Mirror::default()),
      pose(&[], 0.0, Mirror::default())
    );
  }
}

#[test]
fn reads_a_presets_name_and_the_entrance_older_styles_saved() {
  assert_eq!(motion("pop".into()), Ok(presets::named("pop").unwrap()));
  for old in ["none", "words"] {
    let style: CaptionStyle =
      serde_json::from_value(serde_json::json!({ "entrance": old })).unwrap();
    assert_eq!(style.motion, CaptionMotion::default());
  }
  let popped: CaptionStyle =
    serde_json::from_value(serde_json::json!({ "entrance": "pop" })).unwrap();
  assert_eq!(popped.motion, presets::named("pop").unwrap());
  assert!(motion("wobble".into()).unwrap_err().contains("`wobble`"));
}

#[test]
fn turns_away_a_property_this_version_cannot_draw_by_name() {
  let error = motion(serde_json::json!({
    "caption": [{ "property": "rotation", "keyframes": [] }]
  }))
  .unwrap_err();
  assert!(error.contains("`rotation`"), "{error}");
  assert!(error.contains("can't draw"), "{error}");
}

#[test]
fn every_property_is_written_by_its_own_name() {
  for property in CaptionProperty::ALL {
    assert_eq!(
      serde_json::to_value(property).unwrap(),
      property.spec().name
    );
    let (min, max) = property.spec().range;
    assert!((min..=max).contains(&property.spec().rest));
  }
}

#[test]
fn keeps_a_motion_within_its_limits() {
  let keyframes: Vec<_> = (0..MAX_KEYFRAMES + 2)
    .map(|at| serde_json::json!({ "atMs": at * 100, "value": 0.5 }))
    .collect();
  let wild = motion(serde_json::json!({
    "word": [
      { "property": "scale", "keyframes": keyframes },
      { "property": "scale", "keyframes": [] },
      { "property": "opacity", "keyframes": [
        { "atMs": 9000, "value": 3 },
        { "atMs": 10, "value": -1 }
      ] }
    ]
  }))
  .unwrap();
  let kept = wild.normalized();
  assert_eq!(kept.word.len(), 2);
  assert_eq!(kept.word[0].keyframes.len(), MAX_KEYFRAMES);
  let opacity = &kept.word[1].keyframes;
  assert_eq!((opacity[0].at_ms, opacity[0].value), (10.0, 0.0));
  assert_eq!((opacity[1].at_ms, opacity[1].value), (2_000.0, 1.0));
  // A template is held to them rather than drawn clamped.
  let manifest = serde_json::json!({
    "formatVersion": FORMAT_VERSION,
    "id": "wild",
    "name": "Wild",
    "style": { "motion": { "word": [{ "property": "opacity", "keyframes": [{ "atMs": 9000, "value": 1 }] }] } }
  });
  assert!(parse_manifest(&manifest.to_string(), CaptionTemplateSource::User).is_err());
}

#[test]
fn holds_before_its_first_keyframe_and_after_its_last() {
  let sliding = motion(serde_json::json!({
    "caption": [{ "property": "offsetY", "keyframes": [
      { "atMs": 100, "value": 0.5 },
      { "atMs": 300, "value": 0, "easing": "ease-out" }
    ] }]
  }))
  .unwrap();
  let at = |ms| pose(&sliding.caption, ms, Mirror::default()).get(CaptionProperty::OffsetY);
  assert_eq!(at(0.0), 0.5);
  assert_eq!(at(1_000.0), 0.0);
  assert!(
    at(200.0) < 0.25,
    "eased out, more than half-way at half the time"
  );
}

#[test]
fn the_presets_file_is_in_the_current_format() {
  let (version, names) = presets::bundled();
  assert_eq!(version, FORMAT_VERSION);
  assert!(names.contains(&"fade") && names.contains(&"pop"));
}
