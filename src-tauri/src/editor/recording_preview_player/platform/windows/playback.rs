// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows' playback decoders: one Media Foundation reader per pane, each on
//! a thread of its own so the panes decode every tick at the same time.

use std::path::PathBuf;

use super::pane_decoder::PaneDecoder;
use crate::editor::recording_preview_player::platform::playback::PaneDecoders;
use crate::editor::recording_preview_player::PlayerSources;
use screenwide_export::decode::windows::gpu_decoder::GpuFrame;

pub(in crate::editor::recording_preview_player::platform) struct Panes {
  /// Each pane's index, the source time it ends at, and its decoder.
  panes: Vec<(usize, u64, PaneDecoder)>,
}

impl PaneDecoders for Panes {
  type Frame = GpuFrame;
  // Each decoder's sample owns a pooled DXGI surface, retained until its
  // tick has been submitted.
  const HOLDS_SAMPLES: bool = true;

  fn frames_at(&mut self, position_ms: u64) -> Result<[Option<GpuFrame>; 2], ()> {
    // Every pane is asked before any is waited on, so they decode at once.
    for (_, end_ms, pane) in &self.panes {
      if position_ms < *end_ms && !pane.request(position_ms) {
        return Err(());
      }
    }
    let mut frames = [None, None];
    for (index, end_ms, pane) in &self.panes {
      if position_ms >= *end_ms {
        continue;
      }
      if let Some(slot) = frames.get_mut(*index) {
        *slot = pane.frame().map_err(|_| ())?;
      }
    }
    Ok(frames)
  }
}

/// What opens the playback decoders at `start_ms` on the playback thread.
/// Decode runs at full size: the scene composes at the output's size.
pub(in crate::editor::recording_preview_player::platform) fn open(
  sources: &PlayerSources,
  _factors: (f64, f64),
  start_ms: u64,
) -> Result<impl FnOnce() -> Result<Panes, String> + Send + 'static, String> {
  let mut paths: Vec<(usize, PathBuf, u64)> =
    vec![(0, sources.screen_path.clone(), sources.duration_ms)];
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
  Ok(move || {
    let mut panes = Vec::with_capacity(paths.len());
    for (index, path, end_ms) in paths {
      panes.push((
        index,
        end_ms,
        PaneDecoder::spawn(path, start_ms, surface.clone())?,
      ));
    }
    for (_, _, pane) in &panes {
      pane.opened()?;
    }
    Ok(Panes { panes })
  })
}
