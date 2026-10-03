// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each exported frame composed on the shared GPU device. AVFoundation reads
//! and writes the frames' buffers; this draws one into the next, and says
//! when the GPU is done with it.

use std::collections::VecDeque;

use cidre::cv;

use super::super::super::frame_annotations::ExportAnnotations;
use super::super::super::frame_scene::ExportScenes;
use super::frame_camera::drawn_camera;
use super::frame_grid::{grid_frame, grid_index};
use super::frame_placement::{carried_cursor, on_canvas};
use super::*;
use crate::editor::cursor_effects::GpuCursor;
use crate::editor::keyboard_effects::KeyboardOverlay;
use crate::editor::media_preview::BakeGeometry;
use crate::editor::preview_platform::annotation_gpu::prepared_arrows;
use crate::editor::preview_platform::compositor::video_planes::VideoPlanes;
use crate::editor::preview_platform::compositor::{
  ComposedFrame, Compositor, CursorArtwork, NativeCursors, SourceTexture, FORMAT,
};
use crate::gpu::macos::buffer_plane_texture;
use crate::gpu::Gpu;
use crate::screenshots::ScreenshotOutputSettings;

pub(super) struct FrameComposer<'a> {
  gpu: &'static Gpu,
  compositor: Compositor,
  planes: VideoPlanes,
  settings: &'a ScreenshotOutputSettings,
  annotations: ExportAnnotations,
  cursors: &'a [Option<GpuCursor>],
  keyboards: &'a [KeyboardOverlay],
  /// Where the camera sits, whether it casts a shadow, and whether it is
  /// drawn over the screen's annotations.
  camera: Option<(BakeGeometry, bool, bool)>,
  /// The scenes that move the screen and the camera frame by frame.
  scenes: ExportScenes,
  /// The screen's decoded size, which places the cursors the scenes move.
  source_size: (u32, u32),
  /// The screen frame as the canvas samples it, made from its planes.
  source: Option<SourceTexture>,
  canvas: wgpu::TextureView,
  /// Frames submitted and not yet waited for, oldest first, by token.
  submitted: VecDeque<(u64, wgpu::SubmissionIndex)>,
  next_token: u64,
}

impl<'a> FrameComposer<'a> {
  pub(super) fn new<'r: 'a>(
    request: &CursorExportRequest<'r>,
    timeline: Option<&'a macos::native::CursorTimeline>,
    keyboard_timeline: Option<&'a macos::native::KeyboardTimeline>,
  ) -> Result<Self, String> {
    let gpu = crate::gpu::shared()?;
    let mut compositor = Compositor::new(gpu, NativeCursors::none())?;
    if let Some(timeline) = timeline.filter(|timeline| !timeline.artworks.is_empty()) {
      let artworks = timeline
        .artworks
        .iter()
        .map(CursorArtwork::from)
        .collect::<Vec<_>>();
      compositor.use_cursor_artworks(&artworks)?;
    }
    let (width, height) = crate::screenshots::output_dimensions(request.output)?;
    let canvas = gpu
      .device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenwide export canvas"),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default());
    let camera = request
      .camera
      .map(|(_, options)| {
        Ok::<_, String>((
          on_canvas(media_preview::bake_geometry(options)?, request.output),
          options.camera_drop_shadow,
          request.camera_on_top,
        ))
      })
      .transpose()?;
    let annotations = ExportAnnotations::for_request(request);
    let scenes = ExportScenes::for_request(request);
    Ok(Self {
      gpu,
      planes: VideoPlanes::new(gpu),
      compositor,
      settings: request.output,
      annotations,
      cursors: timeline.map_or(&[][..], |timeline| timeline.frames.as_slice()),
      keyboards: keyboard_timeline.map_or(&[][..], |timeline| timeline.frames.as_slice()),
      camera,
      scenes,
      source_size: (request.width, request.height),
      source: None,
      canvas,
      submitted: VecDeque::new(),
      next_token: 0,
    })
  }

  /// Draws `frame` into its destination and submits it, answering the token
  /// `wait` takes once the destination is to be read.
  pub(super) fn compose(&mut self, frame: &ExportFrame) -> Result<u64, String> {
    let gpu = self.gpu;
    let screen = unsafe { pixel_buffer(frame.screen) }
      .ok_or_else(|| "The export frame has no picture".to_owned())?;
    let destination = unsafe { pixel_buffer(frame.destination) }
      .ok_or_else(|| "The export frame has nowhere to be drawn".to_owned())?;
    let view = |texture: wgpu::Texture| texture.create_view(&Default::default());
    let luma = buffer_plane_texture(gpu, screen, 0, wgpu::TextureFormat::R8Unorm, "screen luma")?;
    let size = (luma.width(), luma.height());
    let chroma = buffer_plane_texture(
      gpu,
      screen,
      1,
      wgpu::TextureFormat::Rg8Unorm,
      "screen chroma",
    )?;
    let source = self.source(size);
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide export frame in"),
      });
    self
      .planes
      .decode(gpu, &mut encoder, &view(luma), &view(chroma), &source.view);
    gpu.queue.submit([encoder.finish()]);
    let source_ms = (frame.source_us + 500) / 1_000;
    // A frame inside a scene is drawn on its own arrangement; every other
    // frame keeps the request's composition and the camera placed once.
    let arranged = self.scenes.at(self.settings, source_ms, frame.frame_ms);
    let settings = arranged
      .as_ref()
      .map_or(self.settings, |(settings, _)| settings);
    let placement = match (&arranged, self.camera) {
      (Some((settings, Some(options))), Some((_, shadow, on_top))) => Some((
        on_canvas(media_preview::bake_geometry(*options)?, settings),
        shadow,
        on_top,
      )),
      _ => self.camera,
    };
    let annotations = self.annotations.with_camera_at(
      source_ms,
      frame.frame_ms,
      settings,
      arranged.as_ref().and_then(|(_, options)| *options),
    );
    let camera = unsafe { pixel_buffer(frame.camera) }
      .zip(placement)
      .map(|(pixels, placement)| {
        let drawn = drawn_camera(&self.compositor, pixels, annotations.camera.as_ref())?;
        Ok::<_, String>((drawn, placement))
      })
      .transpose()?;
    let prepared = prepared_arrows(&annotations.screen, source.size, settings, None, None, None)?;
    let grid = grid_index(frame.source_us);
    self.compositor.draw_with_camera(
      &self.canvas,
      &source,
      settings,
      ComposedFrame {
        cursor: match (&arranged, grid_frame(self.cursors, grid).flatten()) {
          (Some((arranged, _)), Some(cursor)) => Some(carried_cursor(
            cursor,
            self.source_size,
            self.settings,
            arranged,
          )?),
          (_, cursor) => cursor,
        },
        keyboard: grid_frame(self.keyboards, grid),
        foreground_only: false,
        seconds: frame.source_us as f64 / 1_000_000.0,
      },
      camera
        .as_ref()
        .map(|(camera, (geometry, shadow, on_top))| (camera, *geometry, *shadow, *on_top)),
      None,
      &prepared,
    )?;
    let luma = buffer_plane_texture(gpu, destination, 0, wgpu::TextureFormat::R8Unorm, "luma")?;
    let chroma =
      buffer_plane_texture(gpu, destination, 1, wgpu::TextureFormat::Rg8Unorm, "chroma")?;
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide export frame out"),
      });
    self
      .planes
      .encode(gpu, &mut encoder, &self.canvas, &view(luma), &view(chroma));
    let token = self.next_token;
    self.next_token += 1;
    self
      .submitted
      .push_back((token, gpu.queue.submit([encoder.finish()])));
    Ok(token)
  }

  /// Blocks until the GPU has finished the frame `token` names, and every
  /// frame submitted before it.
  pub(super) fn wait(&mut self, token: u64) -> Result<(), String> {
    let Some(position) = self.submitted.iter().position(|(sent, _)| *sent == token) else {
      return Err("The export waited on a frame it never submitted".to_owned());
    };
    let (_, submission) = self
      .submitted
      .drain(..=position)
      .next_back()
      .expect("the drained range holds the frame");
    self
      .gpu
      .device
      .poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: None,
      })
      .map(|_| ())
      .map_err(|error| format!("The GPU did not finish an exported frame: {error}"))
  }

  /// The screen frame's canvas source, made again only if the decoded size
  /// changes.
  fn source(&mut self, size: (u32, u32)) -> SourceTexture {
    if let Some(source) = self.source.as_ref().filter(|source| source.size == size) {
      return source.clone();
    }
    let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide export source"),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: FORMAT,
      // Redactions copy it before they draw over the copy.
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let source = SourceTexture {
      size,
      view: texture.create_view(&Default::default()),
      texture,
      picture: None,
    };
    self.source = Some(source.clone());
    source
  }
}

/// One frame the export loop hands over: its decoded screen, its camera
/// frame if it has one, the buffer it is drawn into, and where it falls in
/// the source.
#[repr(C)]
pub(super) struct ExportFrame {
  pub(super) screen: *mut std::ffi::c_void,
  pub(super) camera: *mut std::ffi::c_void,
  pub(super) destination: *mut std::ffi::c_void,
  pub(super) source_us: u64,
  /// How much output time one frame covers, which a moving annotation's
  /// exposure is measured over.
  pub(super) frame_ms: f32,
}

/// `pointer`, a `CVPixelBufferRef` the caller holds for the frame, if any.
unsafe fn pixel_buffer<'b>(pointer: *mut std::ffi::c_void) -> Option<&'b cv::PixelBuf> {
  unsafe { pointer.cast::<cv::PixelBuf>().as_ref() }
}
