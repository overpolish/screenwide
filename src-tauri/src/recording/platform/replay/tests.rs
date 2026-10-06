// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drives a real replay writer and VideoToolbox at real speed, saves clips
//! from it and probes the movies. Ignored by default, like the recording
//! writer's encoder tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;
use std::time::Duration;

use super::super::media::VideoEncoder;
use super::super::output::{AudioSample, CaptureStats, Command};
use super::super::tests::synthetic_frame;
use super::mux::write_movie;
use super::*;

const MS: i64 = 1_000_000;
const SOURCE_EPOCH: i64 = 1_000_000_000_000;
const FRAME: Duration = Duration::from_millis(33);
const LENGTH: Duration = Duration::from_secs(2);

struct Running {
  commands: SyncSender<Command>,
  producer: JoinHandle<()>,
  stop: Arc<AtomicBool>,
  writer: JoinHandle<()>,
}

/// A replay buffer of `LENGTH` fed 640x480 frames and stereo audio at real
/// speed, until `quiet_after` when the screen stops changing.
fn start(quiet_after: Option<Duration>) -> Running {
  let (commands, inbox) = std::sync::mpsc::sync_channel(8);
  let (ready, readied) = std::sync::mpsc::channel();
  let writer = std::thread::spawn(move || {
    let writer = ReplayWriter::new(ReplayWriterConfig {
      encoder: Some(VideoEncoder::H264),
      fps: 30,
      height: 480,
      keyframes: KeyframeLink::new(),
      length: LENGTH,
      microphone_format: None,
      on_failure: Arc::new(|reason| println!("failure reported: {reason}")),
      primary_video: true,
      stats: Arc::new(CaptureStats::default()),
      system_audio: true,
      timeline_origin: Arc::new(OnceLock::new()),
      width: 640,
    })
    .expect("a replay writer");
    ready.send(()).unwrap();
    let (first_frame, _first_framed) = std::sync::mpsc::channel();
    writer.run(&inbox, &first_frame);
  });
  readied.recv().unwrap();

  let stop = Arc::new(AtomicBool::new(false));
  let producer = {
    let commands = commands.clone();
    let stop = Arc::clone(&stop);
    std::thread::spawn(move || {
      let base = Instant::now();
      let mut tick = 0u32;
      while !stop.load(Ordering::Acquire) {
        let elapsed = FRAME * tick;
        let source_ns = SOURCE_EPOCH + i64::try_from(elapsed.as_nanos()).unwrap();
        if quiet_after.is_none_or(|quiet| elapsed < quiet) {
          let _ = commands.send(Command::Frame(synthetic_frame(
            640,
            480,
            source_ns,
            Instant::now(),
          )));
        }
        let _ = commands.send(Command::SystemAudio(AudioSample {
          samples: vec![0.1; 1_584 * 2],
          source_ns,
          wall: Instant::now(),
        }));
        tick += 1;
        if let Some(remaining) = (base + FRAME * tick).checked_duration_since(Instant::now()) {
          std::thread::sleep(remaining);
        }
      }
    })
  };
  Running {
    commands,
    producer,
    stop,
    writer,
  }
}

impl Running {
  fn take(&self, since_ns: Option<i64>) -> ClipTake {
    let (reply, replies) = std::sync::mpsc::channel();
    self
      .commands
      .send(Command::Replay(ClipRequest {
        at: Instant::now(),
        from: ClipFrom::Latest {
          length_ns: i64::try_from(LENGTH.as_nanos()).unwrap(),
          since_ns,
        },
        reply,
      }))
      .unwrap();
    replies.recv().unwrap().expect("a clip")
  }

  fn finish(self) {
    self.stop.store(true, Ordering::Release);
    self.producer.join().unwrap();
    self.commands.send(Command::Cancel).unwrap();
    self.writer.join().unwrap();
  }
}

fn movie(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join("screenwide-tests");
  std::fs::create_dir_all(&directory).unwrap();
  let path = directory.join(format!("{name}.mov"));
  let _ = std::fs::remove_file(&path);
  path
}

/// The movie's length in milliseconds, and whether every frame decodes with
/// video decode times that strictly increase.
fn probe(path: &Path) -> (u64, bool) {
  let run = |args: &[&str]| {
    std::process::Command::new("ffprobe")
      .args(["-v", "error"])
      .args(args)
      .args(["-of", "csv=p=0"])
      .arg(path)
      .output()
      .expect("ffprobe runs")
  };
  let duration = run(&["-show_entries", "format=duration"]);
  let seconds: f64 = String::from_utf8_lossy(&duration.stdout)
    .trim()
    .parse()
    .unwrap();
  let frames = run(&["-show_entries", "frame=pts"]);
  let packets = run(&["-select_streams", "v", "-show_entries", "packet=dts"]);
  let dts: Vec<i64> = String::from_utf8_lossy(&packets.stdout)
    .lines()
    .filter_map(|line| line.trim().parse().ok())
    .collect();
  let increasing = dts.windows(2).all(|pair| pair[0] < pair[1]);
  (
    (seconds * 1_000.0).round() as u64,
    frames.status.success() && frames.stderr.is_empty() && increasing,
  )
}

fn save(running: &Running, name: &str, since_ns: Option<i64>) -> (i64, i64, u64) {
  let take = running.take(since_ns);
  let path = movie(name);
  let written = write_movie(&path, &take).expect("the clip is written");
  let (probed_ms, decodes) = probe(&path);
  println!(
    "{name}: {}ms..{}ms written={}ms probed={probed_ms}ms decodes={decodes}",
    take.start_ns / MS,
    take.end_ns / MS,
    written.duration_ms
  );
  assert!(decodes, "{name} does not decode cleanly");
  assert!(written.has_system_audio);
  let expected_ms = u64::try_from((take.end_ns - take.start_ns) / MS).unwrap();
  assert!(
    probed_ms.abs_diff(expected_ms) < 100,
    "{name} lasts {probed_ms}ms for a {expected_ms}ms clip"
  );
  (take.start_ns, take.end_ns, probed_ms)
}

#[test]
#[ignore = "drives a real encoder at real speed; run with --ignored"]
fn each_save_holds_only_what_is_new_and_no_more_than_the_length() {
  let running = start(None);

  std::thread::sleep(Duration::from_millis(1_500));
  let (first_start, first_end, _) = save(&running, "replay-first", None);
  assert_eq!(first_start, 0);

  // Three seconds on: more than the length has passed, so the clip is capped
  // to the length, give or take the keyframe it has to start on.
  std::thread::sleep(Duration::from_millis(3_000));
  let (second_start, second_end, second_ms) = save(&running, "replay-second", Some(first_end));
  assert!(second_start > first_end);
  assert!(
    (2_000..2_600).contains(&second_ms),
    "a capped clip lasts {second_ms}ms"
  );

  // Half a second on: only what is new, starting exactly where the last
  // save ended.
  std::thread::sleep(Duration::from_millis(500));
  let (third_start, _, third_ms) = save(&running, "replay-third", Some(second_end));
  assert_eq!(third_start, second_end);
  assert!(
    (400..700).contains(&third_ms),
    "a follow-on clip lasts {third_ms}ms"
  );

  running.finish();
}

#[test]
#[ignore = "drives a real encoder at real speed; run with --ignored"]
fn a_quiet_screen_still_gives_a_clip_of_about_the_length() {
  let running = start(Some(Duration::from_secs(1)));

  std::thread::sleep(Duration::from_millis(4_500));
  let (start, _, probed_ms) = save(&running, "replay-quiet", None);
  // Frames stopped at one second; the refreshed keyframes keep the clip from
  // reaching back to them.
  assert!(
    start >= 1_500 * MS,
    "the clip reaches back to {}ms",
    start / MS
  );
  assert!(
    (2_000..3_100).contains(&probed_ms),
    "a quiet clip lasts {probed_ms}ms"
  );

  running.finish();
}
