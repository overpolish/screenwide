// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording scene drawn the way an export draws a frame: the scenes
//! arrange the canvas (`screenwide_export::scenes`), the cursor is placed on
//! it (`placed_cursor`), and the editor's compositor composes the picture and
//! the cursor in one pass with its own shaders. The pointer is a scripted
//! cursor track, and the zooms are the ones auto zoom plans from it.

use std::time::Duration;

use screenwide_auto_zoom::{plan_records, VisibleArea};
use screenwide_compositor::annotation_gpu::prepared_arrows;
use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_compositor::compositor::{ComposedFrame, Compositor, SourceTexture};
use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings};
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::RecordingSceneClip;

use super::clock::frame_index;
use super::content::{canvas_settings, canvas_texture, Pictures};
use super::script::{cursor_records, keyboard_records};
use super::{PointerScript, PreviewScene};

/// The rate a recording is made at unless a scene says otherwise.
const RECORDING_FPS: u32 = 60;

/// The moment of the recording one drawn frame shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moment {
  pub source_ms: u64,
  /// How much recording time the frame covers, which motion is blurred over.
  pub frame_ms: f32,
}

/// The scripted pointer as the recording's tracks: its cursor, hidden while
/// the script types as a recording's is, and the zooms auto zoom makes of
/// it and of its typing.
struct Track {
  cursor: CursorCompositor,
  zooms: Vec<RecordingSceneClip>,
  duration_ms: u64,
}

impl Track {
  fn new(script: &PointerScript, points: (f64, f64), pixels: (u32, u32)) -> Result<Self, String> {
    let records = cursor_records(script, points, pixels);
    let keyboard = keyboard_records(script);
    let typing = screenwide_records::keyboard::typing_times_us(&keyboard);
    let duration_ms = u64::from(script.duration_ms.max(1));
    Ok(Self {
      cursor: CursorCompositor::from_records(&records, &typing)?,
      zooms: plan_records(&records, &keyboard, duration_ms, VisibleArea::WHOLE),
      duration_ms,
    })
  }
}

pub struct SceneRenderer {
  compositor: Compositor,
  scene: PreviewScene,
  /// The screen's size in points, which the script is written in.
  points: (f64, f64),
  screen: SourceTexture,
  with_screenwide: Option<SourceTexture>,
  track: Option<Track>,
}

impl SceneRenderer {
  /// `scene` recorded from `pictures`, the screen first, `points` in size,
  /// composed by `compositor`. The app's compositor draws the system's
  /// cursor artwork, which the app loads on the main thread as it starts.
  pub fn new(
    compositor: Compositor,
    scene: PreviewScene,
    pictures: Vec<CapturedImage>,
    points: (f64, f64),
  ) -> Result<Self, String> {
    if scene.is_tool() {
      return Err("A tool's preview is drawn by the tool".to_owned());
    }
    let pictures = Pictures::new(pictures)?;
    let screen = compositor.screenshot_source(&pictures.screen)?;
    let with_screenwide = pictures
      .with_screenwide
      .as_ref()
      .map(|picture| compositor.screenshot_source(picture))
      .transpose()?;
    let mut renderer = Self {
      compositor,
      scene: PreviewScene::Ruler,
      points,
      screen,
      with_screenwide,
      track: None,
    };
    renderer.set_scene(scene)?;
    Ok(renderer)
  }

  /// Plays `scene` from now on, the same kind with other values.
  pub fn set_scene(&mut self, scene: PreviewScene) -> Result<(), String> {
    let needs_second = matches!(scene, PreviewScene::IncludeScreenwide { .. });
    if needs_second && self.with_screenwide.is_none() {
      return Err("This preview needs the screen with Screenwide's windows on it".to_owned());
    }
    let script = match &scene {
      PreviewScene::AutoZoom { script, .. } | PreviewScene::FrameRate { script, .. } => {
        Some(script)
      }
      _ => None,
    };
    let unchanged = match (&self.scene, script) {
      (
        PreviewScene::AutoZoom { script: old, .. } | PreviewScene::FrameRate { script: old, .. },
        Some(new),
      ) => old == new && self.track.is_some(),
      _ => false,
    };
    if !unchanged {
      self.track = script
        .map(|script| Track::new(script, self.points, self.screen.size))
        .transpose()?;
    }
    self.scene = scene;
    Ok(())
  }

  /// How long one pass of the scene runs, in milliseconds; `None` for a
  /// still.
  pub fn duration_ms(&self) -> Option<u64> {
    self.track.as_ref().map(|track| track.duration_ms)
  }

  /// The zooms auto zoom made of the script, timed on the recording.
  pub fn zooms(&self) -> &[RecordingSceneClip] {
    self.track.as_ref().map_or(&[], |track| &track.zooms)
  }

  /// The moment `elapsed` into the loop shows: the frame of a recording at
  /// the scene's rate shown then. Counted in whole frames, so the loop
  /// starts over on a frame rather than between two.
  pub fn moment(&self, elapsed: Duration) -> Moment {
    let fps = match self.scene {
      PreviewScene::FrameRate { fps, .. } => fps.clamp(1, 240),
      _ => RECORDING_FPS,
    };
    let frame = u64::from(fps);
    let index = self.duration_ms().map_or(0, |duration| {
      frame_index(elapsed, fps) % (duration * frame / 1_000).max(1)
    });
    Moment {
      source_ms: index * 1_000 / frame,
      frame_ms: 1_000.0 / fps as f32,
    }
  }

  /// Draws `moment` into `target`, a canvas `canvas` pixels in size in the
  /// compositor's format.
  pub fn draw(
    &self,
    target: &wgpu::TextureView,
    canvas: (u32, u32),
    moment: Moment,
  ) -> Result<(), String> {
    let base = canvas_settings(self.screen.size, self.points, canvas);
    match &self.scene {
      PreviewScene::AutoZoom { on, .. } => self.draw_recording(target, base, moment, *on),
      PreviewScene::FrameRate { .. } => self.draw_recording(target, base, moment, false),
      PreviewScene::IncludeScreenwide { included } => {
        let source = if *included {
          self.with_screenwide.as_ref().unwrap_or(&self.screen)
        } else {
          &self.screen
        };
        self.compose(target, source, &base, moment, None)
      }
      PreviewScene::Ruler | PreviewScene::TextRecognition | PreviewScene::Annotate { .. } => {
        Err("A tool's preview is drawn by the tool".to_owned())
      }
    }
  }

  /// `moment` drawn offscreen, as RGBA.
  pub fn render(&self, canvas: (u32, u32), moment: Moment) -> Result<CapturedImage, String> {
    let gpu = self.compositor.gpu();
    let target = canvas_texture(gpu, canvas);
    self.draw(&target.create_view(&Default::default()), canvas, moment)?;
    let mut rgba = gpu.read_texture(&target)?;
    for pixel in rgba.as_chunks_mut::<4>().0 {
      pixel.swap(0, 2);
    }
    Ok(CapturedImage {
      width: canvas.0,
      height: canvas.1,
      rgba,
    })
  }

  /// The recording at `moment`, as the export draws its frame: arranged by
  /// the zooms where `zoomed`, then the cursor placed on that arrangement.
  fn draw_recording(
    &self,
    target: &wgpu::TextureView,
    base: ScreenshotOutputSettings,
    moment: Moment,
    zoomed: bool,
  ) -> Result<(), String> {
    let mut settings = base;
    let Some(track) = self.track.as_ref() else {
      return self.compose(target, &self.screen, &settings, moment, None);
    };
    if zoomed {
      screenwide_export::scenes::arrange_following(
        &track.zooms,
        &[],
        (moment.source_ms, moment.frame_ms),
        &mut settings,
        None,
        None,
      );
    }
    let cursor = placed_cursor(
      &track.cursor,
      moment.source_ms,
      CursorEffectSettings::default(),
      self.screen.size,
      &settings,
    )?;
    self.compose(target, &self.screen, &settings, moment, cursor)
  }

  fn compose(
    &self,
    target: &wgpu::TextureView,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    moment: Moment,
    cursor: Option<screenwide_cursor_effects::GpuCursor>,
  ) -> Result<(), String> {
    let annotations = prepared_arrows(&[], source.size, settings, None, None, None)?;
    self.compositor.draw_with_camera(
      target,
      source,
      settings,
      ComposedFrame {
        cursor,
        keyboard: None,
        foreground_only: false,
        seconds: moment.source_ms as f64 / 1_000.0,
      },
      None,
      None,
      &annotations,
    )
  }
}
