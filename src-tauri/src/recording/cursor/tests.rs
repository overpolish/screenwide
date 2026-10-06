// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn test_event(at: Instant, kind: RawCursorEventKind) -> RawCursorEvent {
  RawCursorEvent {
    appearance: CursorAppearance {
      height: 24.0,
      hotspot_x: 2.0,
      hotspot_y: 3.0,
      style: CursorStyle::Arrow,
      width: 16.0,
    },
    at,
    kind,
    x: 12.0,
    y: 34.0,
  }
}

#[test]
fn reader_keeps_complete_lines_before_a_truncated_tail() {
  let path = std::env::temp_dir().join(format!(
    "screenwide-cursor-reader-{}.jsonl",
    std::process::id()
  ));
  std::fs::write(
    &path,
    concat!(
      "{\"type\":\"header\",\"coordinateSpace\":\"global-logical-points\",",
      "\"platform\":\"test\",\"source\":{\"height\":100.0,\"kind\":\"screen\",",
      "\"platformId\":\"1\",\"videoHeight\":200,\"videoWidth\":200,",
      "\"width\":100.0,\"x\":0.0,\"y\":0.0},",
      "\"timebase\":\"recording-microseconds\",\"version\":1}\n",
      "{\"type\":\"position\",\"timestampUs\":10,\"x\":2.0,\"y\":3.0}\n",
      "{\"type\":\"position\""
    ),
  )
  .unwrap();

  let records = read(&path).unwrap();
  let _ = std::fs::remove_file(path);
  assert_eq!(records.len(), 2);
  assert!(matches!(
    records[1],
    CursorRecord::Position {
      timestamp_us: 10,
      ..
    }
  ));
}

#[test]
fn a_macos_recordings_classic_i_beam_reads_back_as_an_i_beam() {
  let styles = |platform: &str| {
    let path = std::env::temp_dir().join(format!(
      "screenwide-cursor-i-beam-{platform}-{}.jsonl",
      std::process::id()
    ));
    let appearance = |width: u8, height: u8, x: u8, y: u8| {
      format!(
        "{{\"type\":\"appearance\",\"height\":{height}.0,\"hotspotX\":{x}.0,\
         \"hotspotY\":{y}.0,\"style\":\"custom\",\"timestampUs\":0,\"width\":{width}.0}}\n"
      )
    };
    std::fs::write(
      &path,
      format!(
        "{{\"type\":\"header\",\"coordinateSpace\":\"global-logical-points\",\
         \"platform\":\"{platform}\",\"source\":{{\"height\":100.0,\"kind\":\"screen\",\
         \"platformId\":\"1\",\"videoHeight\":200,\"videoWidth\":200,\"width\":100.0,\
         \"x\":0.0,\"y\":0.0}},\"timebase\":\"recording-microseconds\",\"version\":2}}\n{}{}",
        appearance(9, 18, 4, 9),
        appearance(9, 18, 0, 0),
      ),
    )
    .unwrap();
    let records = read(&path).unwrap();
    let _ = std::fs::remove_file(path);
    records
      .into_iter()
      .filter_map(|record| match record {
        CursorRecord::Appearance { style, .. } => Some(style),
        _ => None,
      })
      .collect::<Vec<_>>()
  };
  // Its own size and hotspot name it; any other custom cursor stays custom,
  // and a Windows recording named its cursors by handle already.
  assert_eq!(styles("macos"), [CursorStyle::IBeam, CursorStyle::Custom]);
  assert_eq!(
    styles("windows"),
    [CursorStyle::Custom, CursorStyle::Custom]
  );
}

#[test]
fn initial_snapshot_starts_at_zero_and_motion_keeps_hardware_cadence() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-cursor-stream-{}.jsonl",
    std::process::id()
  ));
  let file = std::fs::File::create(&path).unwrap();
  let mut stream = StreamWriter {
    clock: SidecarClock::new(shared_origin),
    failure: None,
    last_appearance: None,
    last_flush: origin,
    last_move: None,
    last_visibility: None,
    last_position: None,
    output: SidecarOutput::File(std::io::BufWriter::new(file)),
  };

  stream
    .record(test_event(
      origin + Duration::from_millis(50),
      RawCursorEventKind::Snapshot,
    ))
    .unwrap();
  stream
    .record(test_event(
      origin + Duration::from_millis(54),
      RawCursorEventKind::Move,
    ))
    .unwrap();
  stream
    .record(test_event(
      origin + Duration::from_millis(58),
      RawCursorEventKind::Move,
    ))
    .unwrap();
  stream.output.flush().unwrap();
  drop(stream);

  let records = std::fs::read_to_string(&path)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str::<CursorRecord>(line).unwrap())
    .collect::<Vec<_>>();
  let _ = std::fs::remove_file(path);
  assert!(matches!(
    records.as_slice(),
    [
      CursorRecord::Appearance {
        timestamp_us: 0,
        ..
      },
      CursorRecord::Position {
        timestamp_us: 0,
        ..
      },
      CursorRecord::Position {
        timestamp_us: 58_000,
        ..
      }
    ]
  ));
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "requires an interactive Windows desktop"]
fn windows_sampler_writes_a_live_semantic_snapshot() {
  let origin = Arc::new(OnceLock::new());
  origin.set(Instant::now()).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-windows-cursor-live-{}.jsonl",
    std::process::id()
  ));
  let recorder = CursorRecorder::start(
    path,
    origin,
    CursorSource {
      height: 1_080.0,
      kind: CursorSourceKind::Screen,
      platform_id: "test".to_owned(),
      video_height: 1_080,
      video_width: 1_920,
      width: 1_920.0,
      x: 0.0,
      y: 0.0,
    },
    true,
  )
  .unwrap();
  std::thread::sleep(Duration::from_millis(40));
  let path = recorder.stop().unwrap();
  let records = read(&path).unwrap();
  let _ = std::fs::remove_file(path);
  assert!(matches!(
    records.first(),
    Some(CursorRecord::Header {
      coordinate_space,
      platform,
      ..
    }) if coordinate_space == "global-physical-pixels" && platform == "windows"
  ));
  assert!(records
    .iter()
    .any(|record| matches!(record, CursorRecord::Appearance { .. })));
  assert!(records
    .iter()
    .any(|record| matches!(record, CursorRecord::Position { .. })));
}

#[test]
fn visibility_records_exact_landings_and_keeps_hidden_button_releases() {
  let origin = Instant::now();
  let shared_origin = Arc::new(OnceLock::new());
  shared_origin.set(origin).unwrap();
  let path = std::env::temp_dir().join(format!(
    "screenwide-cursor-visibility-{}.jsonl",
    std::process::id()
  ));
  let mut stream = StreamWriter {
    clock: SidecarClock::new(shared_origin),
    failure: None,
    last_appearance: None,
    last_flush: origin,
    last_move: None,
    last_visibility: None,
    last_position: None,
    output: SidecarOutput::File(std::io::BufWriter::new(
      std::fs::File::create(&path).unwrap(),
    )),
  };
  stream
    .record(test_event(origin, RawCursorEventKind::Snapshot))
    .unwrap();
  stream.record_visibility(10_000, false, None).unwrap();
  stream
    .record(test_event(
      origin + Duration::from_millis(20),
      RawCursorEventKind::Move,
    ))
    .unwrap();
  stream
    .record(test_event(
      origin + Duration::from_millis(30),
      RawCursorEventKind::Button {
        button: CursorButton::Left,
        click_count: 1,
        state: ButtonState::Up,
      },
    ))
    .unwrap();
  stream
    .record_visibility(40_000, true, Some((900.0, 800.0)))
    .unwrap();
  stream.output.flush().unwrap();
  let records: Vec<CursorRecord> = std::fs::read_to_string(&path)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  std::fs::remove_file(path).unwrap();
  assert!(records.iter().any(|record| matches!(
    record,
    CursorRecord::Button {
      state: ButtonState::Up,
      ..
    }
  )));
  assert!(!records.iter().any(|record| matches!(
    record,
    CursorRecord::Position {
      timestamp_us: 20_000,
      ..
    }
  )));
  assert!(matches!(
    records.last().unwrap(),
    CursorRecord::Visibility {
      timestamp_us: 40_000,
      visible: true,
      x: 900.0,
      y: 800.0
    }
  ));
}
