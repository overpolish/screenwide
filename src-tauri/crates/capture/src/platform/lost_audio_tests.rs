// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Range;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::media::VideoEncoder;
use super::tests::synthetic_frame;
use super::writer::{Container, VideoSource, Writer, WriterConfig};
use super::{CaptureStats, Command, MicrophoneBuffer, MicrophoneFormat, NANOS_PER_MS};

const FORMAT: MicrophoneFormat = MicrophoneFormat {
  channels: 1,
  sample_rate: 48_000,
};
const LENGTH_MS: u64 = 3_000;
/// Microphone buffers that never reach the writer, as a busy machine loses
/// them.
const LOST_MS: Range<u64> = 1_000..1_600;
const BUFFER_MS: u64 = 10;
const FRAME_EVERY_MS: u64 = 30;

/// Records a screen with a microphone whose buffers stop arriving for a
/// while, then asks the movie how long its microphone track plays: as long
/// as the recording, or short by what was lost, which is the voice running
/// ahead of the picture from that point on.
#[test]
#[ignore = "drives a real encoder at real speed; run with --ignored"]
fn lost_microphone_audio_is_kept_as_silence() {
  let directory = std::env::temp_dir().join("screenwide-tests");
  std::fs::create_dir_all(&directory).unwrap();
  let path = directory.join("lost-microphone-audio.mov");
  let _ = std::fs::remove_file(&path);

  let stats = Arc::new(CaptureStats::default());
  let writer = Writer::new(WriterConfig {
    path: path.clone(),
    width: 640,
    height: 480,
    fps: 30,
    encoder: VideoEncoder::H264,
    system_audio: false,
    microphone_format: Some(FORMAT),
    stats: Arc::clone(&stats),
    on_failure: Arc::new(|reason| println!("failure reported: {reason}")),
    container: Container::quicktime_fragmented(),
    primary_video: true,
    source: VideoSource::Screen,
    timeline_origin: Arc::new(OnceLock::new()),
  })
  .expect("a writer");
  let base = writer.base;
  let (commands, inbox) = mpsc::channel();
  let (first_frame, _first_framed) = mpsc::channel();
  let (reply, replies) = mpsc::channel();

  let producer = std::thread::spawn(move || {
    let frames = usize::try_from(u64::from(FORMAT.sample_rate) * BUFFER_MS / 1_000).unwrap();
    let mut at = 0;
    while at < LENGTH_MS {
      if at % FRAME_EVERY_MS == 0 {
        let frame = synthetic_frame(
          640,
          480,
          1_000_000_000_000 + at as i64 * 1_000_000,
          Instant::now(),
        );
        commands.send(Command::Frame(frame)).unwrap();
      }
      if !LOST_MS.contains(&at) {
        let buffer = MicrophoneBuffer {
          captured_at: Instant::now(),
          samples: vec![0.0; frames],
        };
        commands.send(Command::Microphone(buffer)).unwrap();
      }
      at += BUFFER_MS;
      if let Some(wait) = (base + Duration::from_millis(at)).checked_duration_since(Instant::now())
      {
        std::thread::sleep(wait);
      }
    }
    commands
      .send(Command::Stop {
        at: Instant::now(),
        reply,
      })
      .unwrap();
  });

  writer.run(&inbox, &first_frame);
  replies
    .recv()
    .expect("a reply")
    .expect("the movie is saved");
  producer.join().unwrap();

  let probe = std::process::Command::new("ffprobe")
    .args(["-v", "error", "-select_streams", "a:0"])
    .args(["-show_entries", "stream=duration", "-of", "csv=p=0"])
    .arg(&path)
    .output()
    .expect("ffprobe is installed");
  let seconds: f64 = String::from_utf8_lossy(&probe.stdout)
    .trim()
    .parse()
    .expect("the microphone track has a duration");
  let played_ms = seconds * 1_000.0;
  assert!(
    (played_ms - LENGTH_MS as f64).abs() < 100.0,
    "the microphone plays for {played_ms:.0} ms of a {LENGTH_MS} ms recording"
  );
  let silence_ms = stats.microphone_silence_ns.load(Ordering::Relaxed) / NANOS_PER_MS as u64;
  let lost_ms = LOST_MS.end - LOST_MS.start;
  assert!(
    silence_ms.abs_diff(lost_ms) < 50,
    "{silence_ms} ms of silence for {lost_ms} ms lost"
  );
}
