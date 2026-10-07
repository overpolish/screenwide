// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::audio_writer::AudioWriter;
use super::super::replay::{KeyframeLink, ReplayWriterConfig};
use super::super::*;
use super::audio_stream;
use super::microphone_stream;
use super::writer_thread::{spawn_replay_writer, WriterThread};
use super::Sink;
use crate::recording::encoding::FailureReport;
use crate::recording::monitor::RecordingMonitor;
use crate::recording::SystemAudioSelection;

pub(super) async fn begin(
  microphone_id: Option<String>,
  monitor: Arc<RecordingMonitor>,
  on_failure: FailureReport,
  path: PathBuf,
  system_audio: SystemAudioSelection,
  sink: Sink,
) -> Result<CaptureStart, String> {
  let microphone_source = microphone_id
    .as_deref()
    .map(MicrophoneSource::resolve)
    .transpose()?;
  let microphone_format = microphone_source.as_ref().map(MicrophoneSource::format);
  let stats = Arc::new(CaptureStats::default());
  let timeline_origin = Arc::new(OnceLock::new());
  let holds_notes = matches!(sink, Sink::Movie);
  let (commands, first_framed, worker) = match sink {
    Sink::Movie => {
      let (commands, inbox) = mpsc::sync_channel(FRAME_QUEUE_DEPTH);
      let (first_frame, first_framed) = mpsc::channel();
      let writer = AudioWriter::new(
        path,
        system_audio.enabled,
        microphone_format,
        Arc::clone(&stats),
        Arc::clone(&on_failure),
      )?;
      let worker = std::thread::Builder::new()
        .name("screenwide-audio-writer".to_owned())
        .spawn(move || writer.run(&inbox, first_frame))
        .map_err(|error| error.to_string())?;
      (commands, first_framed, worker)
    }
    Sink::Replay { length } => {
      let WriterThread {
        commands,
        first_frame,
        worker,
      } = spawn_replay_writer(
        ReplayWriterConfig {
          encoder: None,
          fps: 0,
          height: 0,
          keyframes: KeyframeLink::new(),
          length,
          microphone_format,
          on_failure: Arc::clone(&on_failure),
          primary_video: false,
          stats: Arc::clone(&stats),
          system_audio: system_audio.enabled,
          timeline_origin: Arc::clone(&timeline_origin),
          width: 0,
        },
        "screenwide-replay-audio-writer",
      )?;
      (commands, first_frame, worker)
    }
  };

  let content = if system_audio.enabled {
    Some(
      sc::ShareableContent::current()
        .await
        .map_err(|error| error.to_string())?,
    )
  } else {
    None
  };
  let output = content.as_ref().map(|_| {
    ScreenOutput::with(ScreenOutputInner {
      commands: commands.clone(),
      monitor: Arc::clone(&monitor),
      stats: Arc::clone(&stats),
    })
  });
  let queue = dispatch::Queue::serial_with_ar_pool();
  let (watch, stream_reports) = stream_recovery::watch();
  let system_audio_streams = audio_stream::create(
    &system_audio,
    content.as_deref(),
    output.as_ref(),
    &queue,
    &watch,
    false,
  )?;
  let microphone =
    microphone_stream::start(microphone_source, holds_notes, &commands, &monitor, &stats)?;
  system_audio_streams.start().await?;
  let begin_at = Instant::now();
  let _ = timeline_origin.set(begin_at);
  commands
    .send(Command::Begin { at: begin_at })
    .map_err(|_| "The audio recording writer stopped during startup".to_owned())?;

  let mut streams = Vec::new();
  system_audio_streams.append_to(&mut streams);
  Ok(CaptureStart {
    cursor_source: None,
    first_frame: first_framed,
    session: CaptureSession {
      camera: None,
      commands,
      microphone,
      objects: StreamObjects {
        _output: output,
        desktop: None,
        queue,
        streams: RecoveringStreams::new(watch, stream_reports, streams),
      },
      primary_camera: None,
      worker: Some(worker),
    },
    source_scale_factor: 1.0,
    timeline_origin,
  })
}
