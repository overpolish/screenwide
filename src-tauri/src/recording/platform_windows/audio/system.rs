// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn start_system(
  process_id: Option<u32>,
  path: PathBuf,
  origin: Arc<OnceLock<Instant>>,
  paused: Arc<AtomicBool>,
  monitor: Arc<RecordingMonitor>,
  on_failure: FailureReport,
) -> Result<SystemCapture, String> {
  let sink = RawSink::start(path, SYSTEM_SAMPLE_RATE, SYSTEM_CHANNELS, origin)?;
  let sender = sink.sender()?;
  let stop = Arc::new(AtomicBool::new(false));
  let thread_stop = Arc::clone(&stop);
  let (ready_tx, ready_rx) = mpsc::sync_channel(1);
  let thread = thread::Builder::new()
    .name("screenwide-windows-system-audio".to_owned())
    .spawn(move || {
      capture_system(
        process_id,
        thread_stop,
        paused,
        sender,
        monitor,
        ready_tx,
        on_failure,
      );
    })
    .map_err(|error| error.to_string())?;
  ready_rx
    .recv_timeout(Duration::from_secs(10))
    .map_err(|_| "Timed out starting Windows system audio".to_owned())??;
  Ok(SystemCapture {
    sink,
    stop,
    thread: Some(thread),
  })
}

pub(super) fn capture_system(
  process_id: Option<u32>,
  stop: Arc<AtomicBool>,
  paused: Arc<AtomicBool>,
  sender: mpsc::SyncSender<Packet>,
  monitor: Arc<RecordingMonitor>,
  ready: mpsc::SyncSender<Result<(), String>>,
  on_failure: FailureReport,
) {
  if initialize_mta().is_err() {
    let _ = ready.send(Err("Could not initialize Windows system audio".to_owned()));
    return;
  }
  let initialized = initialize_system_client(process_id);
  let (audio_client, capture_client, event) = match initialized {
    Ok(value) => {
      let _ = ready.send(Ok(()));
      value
    }
    Err(error) => {
      let _ = ready.send(Err(error));
      wasapi::deinitialize();
      return;
    }
  };
  if let Err(error) =
    capture_system_packets(&capture_client, &event, &stop, &paused, &sender, &monitor)
  {
    on_failure(format!("System audio recording stopped: {error}"));
  }
  let _ = audio_client.stop_stream();
  wasapi::deinitialize();
}

pub(super) fn initialize_system_client(
  process_id: Option<u32>,
) -> Result<(AudioClient, AudioCaptureClient, Handle), String> {
  let format = WaveFormat::new(
    32,
    32,
    &SampleType::Float,
    SYSTEM_SAMPLE_RATE as usize,
    SYSTEM_CHANNELS as usize,
    None,
  );
  let (mut client, direction) = if let Some(process_id) = process_id {
    (
      AudioClient::new_application_loopback_client(process_id, true)
        .map_err(|error| error.to_string())?,
      Direction::Capture,
    )
  } else {
    let device = DeviceEnumerator::new()
      .and_then(|enumerator| enumerator.get_default_device(&Direction::Render))
      .map_err(|error| error.to_string())?;
    (
      device
        .get_iaudioclient()
        .map_err(|error| error.to_string())?,
      Direction::Capture,
    )
  };
  client
    .initialize_client(
      &format,
      &direction,
      &StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: 0,
      },
    )
    .map_err(|error| error.to_string())?;
  let event = client
    .set_get_eventhandle()
    .map_err(|error| error.to_string())?;
  let capture = client
    .get_audiocaptureclient()
    .map_err(|error| error.to_string())?;
  client.start_stream().map_err(|error| error.to_string())?;
  Ok((client, capture, event))
}

pub(super) fn capture_system_packets(
  capture: &AudioCaptureClient,
  event: &Handle,
  stop: &AtomicBool,
  paused: &AtomicBool,
  sender: &mpsc::SyncSender<Packet>,
  monitor: &RecordingMonitor,
) -> Result<(), String> {
  let mut bytes = Vec::new();
  while !stop.load(Ordering::Acquire) {
    while capture
      .get_next_packet_size()
      .map_err(|error| error.to_string())?
      .is_some_and(|frames| frames > 0)
    {
      let frames = capture
        .get_next_packet_size()
        .map_err(|error| error.to_string())?
        .unwrap_or(0) as usize;
      bytes.resize(frames * usize::from(SYSTEM_CHANNELS) * size_of::<f32>(), 0);
      let (read, info) = capture
        .read_from_device(&mut bytes)
        .map_err(|error| error.to_string())?;
      let sample_count = read as usize * usize::from(SYSTEM_CHANNELS);
      let samples = if info.flags.silent {
        vec![0.0; sample_count]
      } else {
        bytes[..sample_count * size_of::<f32>()]
          .chunks_exact(4)
          .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap_or([0; 4])))
          .collect::<Vec<_>>()
      };
      monitor.send_system_audio(&samples);
      if !paused.load(Ordering::Acquire) {
        let duration = Duration::from_secs_f64(read as f64 / f64::from(SYSTEM_SAMPLE_RATE));
        let now = Instant::now();
        let captured_at = now.checked_sub(duration).unwrap_or(now);
        let _ = sender.try_send(Packet {
          captured_at,
          samples,
        });
      }
    }
    let _ = event.wait_for_event(50);
  }
  Ok(())
}
