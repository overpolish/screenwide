// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn event(
  focus: FocusContext,
  is_printable: bool,
  modifiers: Vec<KeyboardModifier>,
) -> RawKeyboardEvent {
  RawKeyboardEvent {
    at: Instant::now(),
    focus,
    kind: RawKeyboardEventKind::KeyDown {
      is_printable,
      is_repeat: false,
    },
    key_code: 9,
    modifiers,
  }
}

fn key_up(key_code: u16) -> RawKeyboardEvent {
  RawKeyboardEvent {
    at: Instant::now(),
    focus: FocusContext::NonText,
    kind: RawKeyboardEventKind::KeyUp,
    key_code,
    modifiers: vec![],
  }
}

#[test]
fn classifier_records_lone_printable_keys_only_outside_text() {
  assert!(StreamWriter::accepts(&event(
    FocusContext::NonText,
    true,
    vec![]
  )));
  assert!(!StreamWriter::accepts(&event(
    FocusContext::Text,
    true,
    vec![]
  )));
  assert!(!StreamWriter::accepts(&event(
    FocusContext::Unknown,
    true,
    vec![]
  )));
}

#[test]
fn focus_changes_fail_closed_for_printable_keys() {
  assert_eq!(
    FocusContext::conservative(FocusContext::NonText, FocusContext::Text),
    FocusContext::Text
  );
  assert_eq!(
    FocusContext::conservative(FocusContext::Text, FocusContext::NonText),
    FocusContext::Text
  );
  assert_eq!(
    FocusContext::conservative(FocusContext::Unknown, FocusContext::NonText),
    FocusContext::Unknown
  );
}

#[test]
fn classifier_records_shortcuts_in_text_but_nothing_secure() {
  let command = vec![KeyboardModifier::Command];
  assert!(StreamWriter::accepts(&event(
    FocusContext::Text,
    true,
    command.clone()
  )));
  assert!(!StreamWriter::accepts(&event(
    FocusContext::Secure,
    true,
    command
  )));
}

#[test]
fn text_context_rejects_printable_shift_and_option_chords() {
  for modifier in [KeyboardModifier::Shift, KeyboardModifier::Option] {
    assert!(!StreamWriter::accepts(&event(
      FocusContext::Text,
      true,
      vec![modifier]
    )));
    assert!(!StreamWriter::accepts(&event(
      FocusContext::Unknown,
      true,
      vec![modifier]
    )));
  }
}

#[test]
fn classifier_records_non_printable_keys_without_exposing_text() {
  assert!(StreamWriter::accepts(&event(
    FocusContext::Text,
    false,
    vec![]
  )));
  assert!(StreamWriter::accepts(&event(
    FocusContext::Unknown,
    false,
    vec![]
  )));
}

#[test]
fn classifier_ignores_key_repeat() {
  let mut repeated = event(FocusContext::NonText, true, vec![]);
  repeated.kind = RawKeyboardEventKind::KeyDown {
    is_printable: true,
    is_repeat: true,
  };
  assert!(!StreamWriter::accepts(&repeated));
}

#[test]
fn physical_modifier_release_wins_over_an_aggregate_flag() {
  assert!(modifier_transition_is_down(false, true));
  assert!(!modifier_transition_is_down(true, true));
  assert!(!modifier_transition_is_down(false, false));
}

#[test]
fn accepted_key_down_and_matching_key_up_have_distinct_timestamps() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-keyboard-v2-{}.jsonl",
    std::process::id()
  ));
  let file = std::fs::File::create(&path).unwrap();
  let mut writer = StreamWriter::new(
    SidecarClock::new(shared_origin),
    SidecarOutput::File(std::io::BufWriter::new(file)),
  );
  let mut down = event(FocusContext::NonText, true, vec![]);
  down.at = origin + Duration::from_millis(10);
  let mut up = key_up(down.key_code);
  up.at = origin + Duration::from_millis(35);
  assert!(writer.record(down).unwrap());
  assert!(writer.record(up).unwrap());
  writer.output.flush().unwrap();
  let lines = std::fs::read_to_string(&path).unwrap();
  let _ = std::fs::remove_file(path);
  assert!(lines.contains("\"type\":\"keyDown\""));
  assert!(lines.contains("\"type\":\"keyUp\""));
  assert!(lines.contains("\"timestampUs\":10000"));
  assert!(lines.contains("\"timestampUs\":35000"));
}

#[test]
fn key_up_without_accepted_key_down_is_discarded() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-keyboard-v2-up-{}.jsonl",
    std::process::id()
  ));
  let file = std::fs::File::create(&path).unwrap();
  let mut writer = StreamWriter::new(
    SidecarClock::new(shared_origin),
    SidecarOutput::File(std::io::BufWriter::new(file)),
  );
  assert!(!writer.record(key_up(99)).unwrap());
  let _ = std::fs::remove_file(path);
}

#[test]
fn a_hidden_shortcut_leaves_no_keys_not_even_its_modifiers() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-keyboard-hidden-{}.jsonl",
    std::process::id()
  ));
  let file = std::fs::File::create(&path).unwrap();
  let mut writer = StreamWriter::new(
    SidecarClock::new(shared_origin),
    SidecarOutput::File(std::io::BufWriter::new(file)),
  );
  // Super+Control+Digit7, which no other test presses.
  set_hidden_shortcuts(&["Super+Control+Digit7".to_owned()]);
  let flags = |key_code: u16, modifier: KeyboardModifier, is_down: bool| RawKeyboardEvent {
    at: Instant::now(),
    focus: FocusContext::NonText,
    kind: RawKeyboardEventKind::FlagsChanged { is_down, modifier },
    key_code,
    modifiers: vec![],
  };
  let press = |key_code: u16, modifiers: Vec<KeyboardModifier>| {
    let mut down = event(FocusContext::NonText, false, modifiers);
    down.key_code = key_code;
    down
  };
  let chord = vec![KeyboardModifier::Command, KeyboardModifier::Control];
  writer
    .record(flags(0x3b, KeyboardModifier::Control, true))
    .unwrap();
  writer
    .record(flags(0x37, KeyboardModifier::Command, true))
    .unwrap();
  writer.record(press(0x1a, chord)).unwrap();
  writer.record(key_up(0x1a)).unwrap();
  writer
    .record(flags(0x37, KeyboardModifier::Command, false))
    .unwrap();
  writer
    .record(flags(0x3b, KeyboardModifier::Control, false))
    .unwrap();
  writer.output.flush().unwrap();
  let hidden = std::fs::read_to_string(&path).unwrap();
  // The same keys as an ordinary shortcut are kept, modifier and all.
  writer
    .record(flags(0x37, KeyboardModifier::Command, true))
    .unwrap();
  writer
    .record(press(0x1a, vec![KeyboardModifier::Command]))
    .unwrap();
  writer.output.flush().unwrap();
  let kept = std::fs::read_to_string(&path).unwrap();
  set_hidden_shortcuts(&[]);
  let _ = std::fs::remove_file(path);
  assert_eq!(hidden, "");
  assert_eq!(kept.lines().count(), 2);
  assert!(kept.contains("\"keyCode\":55"));
  assert!(kept.contains("\"keyCode\":26"));
}

#[test]
fn typing_in_a_field_is_marked_by_time_alone_and_never_in_a_password() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-keyboard-typing-{}.jsonl",
    std::process::id()
  ));
  let file = std::fs::File::create(&path).unwrap();
  let mut writer = StreamWriter::new(
    SidecarClock::new(shared_origin),
    SidecarOutput::File(std::io::BufWriter::new(file)),
  );
  let at = |focus, ms| {
    let mut key = event(focus, true, vec![]);
    key.at = origin + Duration::from_millis(ms);
    key
  };
  // A password typed first leaves nothing, not even when.
  assert!(!writer.record(at(FocusContext::Secure, 0)).unwrap());
  // Keys a quarter of a second apart: one mark per half second of them.
  let marked: Vec<bool> = [100, 350, 600, 850]
    .into_iter()
    .map(|ms| writer.record(at(FocusContext::Text, ms)).unwrap())
    .collect();
  assert_eq!(marked, [true, false, true, false]);
  writer.output.flush().unwrap();
  let lines = std::fs::read_to_string(&path).unwrap();
  let _ = std::fs::remove_file(path);
  assert_eq!(
    lines.lines().collect::<Vec<_>>(),
    [
      r#"{"type":"typing","timestampUs":100000}"#,
      r#"{"type":"typing","timestampUs":600000}"#,
    ]
  );
}

#[test]
fn reader_keeps_complete_lines_before_a_truncated_tail() {
  let path = std::env::temp_dir().join(format!(
    "screenwide-keyboard-reader-{}.jsonl",
    std::process::id()
  ));
  std::fs::write(
    &path,
    concat!(
      "{\"type\":\"header\",\"platform\":\"test\",",
      "\"timebase\":\"recording-microseconds\",\"version\":1}\n",
      "{\"type\":\"shortcut\",\"keyCode\":9,\"modifiers\":[\"command\"],",
      "\"timestampUs\":10}\n",
      "{\"type\":\"shortcut\""
    ),
  )
  .unwrap();

  let records = read(&path).unwrap();
  let _ = std::fs::remove_file(path);
  assert_eq!(records.len(), 2);
  assert!(matches!(
    records[1],
    KeyboardRecord::Shortcut {
      timestamp_us: 10,
      ..
    }
  ));
}
