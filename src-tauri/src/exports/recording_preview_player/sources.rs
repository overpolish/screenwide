// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[derive(Clone)]
pub(super) struct PlayerSources {
  pub(super) audio_tracks: Vec<RecordingAudioTrack>,
  pub(super) camera_duration_ms: Option<u64>,
  pub(super) camera_path: Option<PathBuf>,
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(super) cursor: Option<Arc<CursorCompositor>>,
  #[cfg(target_os = "macos")]
  pub(super) cursor_artworks: Option<Arc<Vec<GpuArtwork>>>,
  pub(super) cursor_settings: Arc<RwLock<CursorEffectSettings>>,
  pub(super) keyboard: Option<Arc<KeyboardCompositor>>,
  pub(super) keyboard_animation_ranges: Arc<RwLock<Vec<TimelineRange>>>,
  pub(super) keyboard_settings: Arc<RwLock<KeyboardEffectSettings>>,
  pub(super) composition_settings: Option<Arc<RwLock<PreviewCompositionSettings>>>,
  pub(super) duration_ms: u64,
  pub(super) frames_per_second: Option<f64>,
  /// Zero when OSCs are hidden, one for the primary pane and two for camera.
  pub(super) layout: RecordingPreviewLayout,
  pub(super) playback_layout: RecordingPreviewLayout,
  /// True while real-time playback owns the surface, so a late still decode
  /// never stomps a playing frame.
  pub(super) playing: Arc<AtomicBool>,
  pub(super) preview_surface: Option<Arc<RecordingPreviewSurface>>,
  pub(super) primary_kind: PrimaryRecordingKind,
  pub(super) screen_path: PathBuf,
}

impl PlayerSources {
  pub(super) fn keyboard_overlay(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    dimensions: (u32, u32),
  ) -> Option<crate::exports::keyboard_effects::KeyboardOverlay> {
    let keyboard = self.keyboard.as_deref()?;
    let ranges = self
      .keyboard_animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    keyboard.evaluate_fitted_with_ranges(
      position_ms,
      settings,
      dimensions,
      (!ranges.is_empty()).then_some(ranges.as_slice()),
    )
  }
}

pub(super) fn sources(
  app: &AppHandle,
  artifact_id: u64,
  settings: Option<&PreviewPlayerSettings>,
) -> Result<PlayerSources, String> {
  sources_with_surface(app, artifact_id, settings, true)
}

pub(super) fn headless_sources(app: &AppHandle, artifact_id: u64) -> Result<PlayerSources, String> {
  sources_with_surface(app, artifact_id, None, false)
}

fn sources_with_surface(
  app: &AppHandle,
  artifact_id: u64,
  settings: Option<&PreviewPlayerSettings>,
  create_surface: bool,
) -> Result<PlayerSources, String> {
  let state = app.state::<ExportState>();
  let (
    audio_tracks,
    camera,
    cursor_path,
    keyboard_path,
    duration_ms,
    height,
    path,
    primary_kind,
    width,
  ) = {
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(ExportArtifact::Recording {
      audio_tracks,
      camera,
      cursor,
      keyboard,
      duration_ms,
      height,
      id,
      path,
      primary_kind,
      width,
      ..
    }) = artifact.as_ref()
    else {
      return Err("There is no recording to preview".to_owned());
    };
    if *id != artifact_id {
      return Err("That recording is no longer waiting to be exported".to_owned());
    }
    (
      audio_tracks.clone(),
      camera.clone(),
      cursor.as_ref().map(|value| value.path.clone()),
      keyboard.as_ref().map(|value| value.path.clone()),
      *duration_ms,
      *height,
      path.clone(),
      *primary_kind,
      *width,
    )
  };
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let frames_per_second =
    super::super::media_preview::recording_info(&path).and_then(|info| info.frames_per_second);
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let frames_per_second = None;
  let camera_size = camera.as_ref().map(|value| (value.width, value.height));
  let primary_pane = match primary_kind {
    PrimaryRecordingKind::Screen => Some((width, height, layout::PreviewPaneKind::Screen)),
    PrimaryRecordingKind::Camera => Some((width, height, layout::PreviewPaneKind::Camera)),
    PrimaryRecordingKind::Audio => None,
  };
  // Creating the native surface synchronously asks the main thread for the
  // export window's NSView/HWND. Never do that while holding the artifact
  // mutex: the main thread may simultaneously be serving a snapshot request
  // that needs the same mutex, which deadlocks crash recovery on startup.
  let preview_surface = if create_surface {
    app
      .get_webview_window(ExportKind::Recording.window_label().as_str())
      .map(|window| RecordingPreviewSurface::from_window(&window).map(Arc::new))
      .transpose()?
  } else {
    None
  };
  let layout = preview_layout(primary_pane, camera_size, height);
  // Native playback decodes every source at its own stored resolution. The
  // presentation surface handles its visual size, so a portrait camera is not
  // needlessly enlarged to the screen track's height before composition.
  let mut playback_layout = layout.clone();
  for pane in &mut playback_layout.panes {
    pane.width = pane.source_width;
    pane.height = pane.source_height;
  }
  playback_layout.width = playback_layout.panes.iter().map(|pane| pane.width).sum();
  playback_layout.height = playback_layout
    .panes
    .iter()
    .map(|pane| pane.height)
    .max()
    .unwrap_or(0);
  let cursor = cursor_path
    .as_ref()
    .map(|path| CursorCompositor::open(path).map(Arc::new))
    .transpose()?;
  let keyboard = keyboard_path
    .as_ref()
    .map(|keyboard_path| {
      let persisted =
        crate::exports::timeline_edit::for_recording(&path, artifact_id).and_then(|(_, edit)| {
          crate::exports::timeline_edit::TimelinePlan::from_edit(&edit, duration_ms)
        });
      let deleted_ids = settings
        .map(|settings| settings.deleted_keyboard_shortcut_ids.clone())
        .unwrap_or_else(|| {
          persisted.as_ref().map_or_else(Vec::new, |plan| {
            plan.deleted_keyboard_shortcut_ids().to_vec()
          })
        });
      let deleted_ranges = settings
        .map(|settings| settings.deleted_keyboard_shortcut_ranges.clone())
        .unwrap_or_else(|| {
          persisted.as_ref().map_or_else(Vec::new, |plan| {
            plan.deleted_keyboard_shortcut_ranges().to_vec()
          })
        });
      let compositor =
        KeyboardCompositor::open_with_deleted(keyboard_path, &deleted_ids, &deleted_ranges)?;
      let positions = settings
        .map(|settings| settings.keyboard_shortcut_positions.as_slice())
        .or_else(|| {
          persisted
            .as_ref()
            .map(|plan| plan.keyboard_shortcut_positions())
        })
        .unwrap_or(&[]);
      compositor.set_shortcut_positions(positions);
      Ok::<_, String>(Arc::new(compositor))
    })
    .transpose()?;
  #[cfg(target_os = "macos")]
  let cursor_artworks = cursor
    .as_ref()
    .map(|_| Arc::new(crate::exports::cursor_effects::gpu_artworks()));
  Ok(PlayerSources {
    audio_tracks,
    camera_duration_ms: camera.as_ref().map(|value| value.duration_ms),
    camera_path: camera.as_ref().map(|value| value.path.clone()),
    cursor,
    #[cfg(target_os = "macos")]
    cursor_artworks,
    composition_settings: settings.map(|settings| {
      Arc::new(RwLock::new(PreviewCompositionSettings {
        bake_camera: settings.bake_camera,
        camera_overlay: settings.camera_overlay,
        recording_output: settings.recording_output.clone(),
      }))
    }),
    cursor_settings: Arc::new(RwLock::new(
      settings.map_or_else(CursorEffectSettings::default, |settings| {
        settings.cursor_effects
      }),
    )),
    keyboard,
    keyboard_animation_ranges: Arc::new(RwLock::new(settings.map_or_else(Vec::new, |settings| {
      animation_timeline_ranges(&settings.playback_ranges)
    }))),
    keyboard_settings: Arc::new(RwLock::new(
      settings.map_or_else(KeyboardEffectSettings::default, |settings| {
        settings.keyboard_effects.normalized()
      }),
    )),
    duration_ms,
    frames_per_second,
    layout,
    playback_layout,
    playing: Arc::new(AtomicBool::new(false)),
    preview_surface,
    primary_kind,
    screen_path: path,
  })
}
