// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_video(
  sources: &PlayerSources,
  _playback_factors: &[f64],
  start_ms: u64,
  playback_rate: f64,
  _still: bool,
  cancelled: Arc<AtomicBool>,
  _child: Arc<Mutex<Option<Child>>>,
  sender: SyncSender<VideoFrame>,
) -> Result<std::thread::JoinHandle<()>, String> {
  if sources.playback_layout.panes.is_empty() {
    return Err("The recording has no video pane".to_owned());
  }
  let mut paths = vec![(0_u32, sources.screen_path.clone(), sources.duration_ms)];
  if let Some(path) = &sources.camera_path {
    paths.push((
      1,
      path.clone(),
      sources.camera_duration_ms.unwrap_or(sources.duration_ms),
    ));
  }
  let surface = sources
    .preview_surface
    .clone()
    .ok_or_else(|| "Windows GPU preview has no native presentation surface".to_owned())?;

  let (startup_tx, startup_rx) = mpsc::sync_channel(1);
  let thread = std::thread::Builder::new()
    .name("recording-preview-video-windows".to_owned())
    .spawn(move || {
      let mut streams = Vec::with_capacity(paths.len());
      for (index, path, duration_ms) in paths {
        let mut reader = match GpuVideoReader::open(&path, start_ms, surface.clone()) {
          Ok(reader) => reader,
          Err(error) => {
            let _ = startup_tx.send(Err(error));
            return;
          }
        };
        match reader.frame_at(start_ms) {
          Ok(_) => {}
          Err(error) => {
            let _ = startup_tx.send(Err(error));
            return;
          }
        }
        streams.push((index, duration_ms, reader));
      }
      let _ = startup_tx.send(Ok(()));
      let mut output_frame = 0_u64;
      while !cancelled.load(Ordering::Acquire) {
        let target_ms = source_position_ms(start_ms, output_frame, playback_rate);
        let mut sent = false;
        for (index, duration_ms, reader) in &mut streams {
          if target_ms >= *duration_ms {
            continue;
          }
          let mut frame = match reader.frame_at(target_ms) {
            Ok(Some(frame)) => frame,
            Ok(None) => continue,
            Err(_) => return,
          };
          // Presentation metadata follows the output tick even when the
          // retained source texture repeats, so overlays still animate at
          // the full output cadence.
          frame.timestamp_ms = target_ms;
          frame.timestamp_100ns = i64::try_from(target_ms)
            .unwrap_or(i64::MAX / 10_000)
            .saturating_mul(10_000);
          // The decoder's sample owns a pooled DXGI surface. Keep it retained
          // until this output tick has submitted the texture; repeated slow-
          // motion ticks safely reuse the retained sample before decoding on.
          let (presented_tx, presented_rx) = mpsc::sync_channel(0);
          let mut frame = VideoFrame {
            presentation_elapsed_ms: presentation_elapsed_ms(output_frame),
            payload: VideoFramePayload::Native {
              frame,
              index: *index,
              presented: Some(presented_tx),
            },
          };
          loop {
            match sender.try_send(frame) {
              Ok(()) => break,
              Err(TrySendError::Full(returned)) => {
                if cancelled.load(Ordering::Acquire) {
                  return;
                }
                frame = returned;
                std::thread::yield_now();
              }
              Err(TrySendError::Disconnected(_)) => return,
            }
          }
          while !cancelled.load(Ordering::Acquire) {
            match presented_rx.recv_timeout(Duration::from_millis(50)) {
              Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
              Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
          }
          sent = true;
        }
        if !sent {
          break;
        }
        output_frame = output_frame.saturating_add(1);
      }
    })
    .map_err(|error| error.to_string())?;
  match startup_rx.recv_timeout(Duration::from_secs(5)) {
    Ok(Ok(())) => Ok(thread),
    Ok(Err(error)) => {
      let _ = thread.join();
      Err(error)
    }
    Err(_) => {
      let _ = thread.join();
      Err("Media Foundation did not open the preview in time".to_owned())
    }
  }
}

pub(crate) fn playback_factors(
  pane_target_sizes: &[(u32, u32)],
  sources: &PlayerSources,
) -> Vec<f64> {
  sources
    .playback_layout
    .panes
    .iter()
    .enumerate()
    .map(|(index, pane)| {
      pane_target_sizes
        .get(index)
        .map(|size| f64::from(size.0.max(16)) / f64::from(pane.source_width.max(1)))
        .unwrap_or(0.5)
        .clamp(0.1, 1.0)
    })
    .collect()
}
