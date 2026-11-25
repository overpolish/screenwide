// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The macOS side of an exported frame: AVFoundation's decoded planes in,
//! the shared [`FrameDraw`] over them, and each format's canvas back out as
//! planes. Every format is drawn from the one screen frame, converted from
//! its planes once.

use std::collections::VecDeque;

use cidre::cv;

use super::super::super::frame_draw::FrameDraw;
use super::*;
use screenwide_compositor::compositor::video_planes::VideoPlanes;
use screenwide_compositor::compositor::{Compositor, SourceTexture, FORMAT};
use screenwide_gpu::macos::{bgra_buffer_texture, buffer_plane_texture};
use screenwide_gpu::Gpu;

/// One format's drawing: its own compositor, what it draws over the screen,
/// and the canvas it draws on.
struct FormatDraw<'a> {
  compositor: Compositor,
  draw: FrameDraw<'a>,
  canvas: wgpu::TextureView,
}

impl<'a> FormatDraw<'a> {
  fn new<'r: 'a>(
    request: &CursorExportRequest<'r>,
    gpu: &std::sync::Arc<Gpu>,
    annotations_within_ms: Option<[u64; 2]>,
  ) -> Result<Self, String> {
    let compositor = Compositor::new(gpu)?;
    compositor.draw_generators_at_half_resolution();
    let (width, height) = screenwide_output::output_dimensions(request.output)?;
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
    Ok(Self {
      draw: FrameDraw::new(request, gpu, annotations_within_ms)?,
      compositor,
      canvas,
    })
  }
}

pub(super) struct FrameComposer<'a> {
  gpu: std::sync::Arc<Gpu>,
  planes: VideoPlanes,
  /// Every format drawn, the loop's own first where it draws one.
  formats: Vec<FormatDraw<'a>>,
  /// Whether the first format is drawn into the loop's own buffer, rather
  /// than every format into buffers of their own.
  draws_loop_buffer: bool,
  /// The screen frame as the canvas samples it, made from its planes.
  source: Option<SourceTexture>,
  /// Frames submitted and not yet waited for, oldest first, by token.
  submitted: VecDeque<(u64, wgpu::SubmissionIndex)>,
  next_token: u64,
}

impl<'a> FrameComposer<'a> {
  /// Draws `requests` from one decoded frame: the first into the loop's own
  /// buffer where `draws_loop_buffer`, the rest into buffers of their own.
  /// `annotations_within_ms` is the stretch a preview draws; see
  /// `ExportAnnotations`.
  pub(super) fn new<'r: 'a>(
    requests: &[&CursorExportRequest<'r>],
    draws_loop_buffer: bool,
    annotations_within_ms: Option<[u64; 2]>,
  ) -> Result<Self, String> {
    let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Heavy)?;
    let formats = requests
      .iter()
      .map(|request| FormatDraw::new(request, &gpu, annotations_within_ms))
      .collect::<Result<Vec<_>, _>>()?;
    Ok(Self {
      planes: VideoPlanes::new(&gpu),
      gpu,
      formats,
      draws_loop_buffer,
      source: None,
      submitted: VecDeque::new(),
      next_token: 0,
    })
  }

  /// Draws `frame` into its destination where the loop's own format is
  /// drawn there, and the other formats into `beside` in their order, and
  /// submits them, answering the token `wait` takes once the destinations
  /// are to be read.
  pub(super) fn compose(
    &mut self,
    frame: &ExportFrame,
    beside: &[&cv::PixelBuf],
  ) -> Result<u64, String> {
    // A handle of its own, so `self` can be borrowed mutably while drawing.
    let gpu = std::sync::Arc::clone(&self.gpu);
    let gpu = &*gpu;
    let screen = unsafe { pixel_buffer(frame.screen) }
      .ok_or_else(|| "The export frame has no picture".to_owned())?;
    let destination = if self.draws_loop_buffer {
      Some(
        unsafe { pixel_buffer(frame.destination) }
          .ok_or_else(|| "The export frame has nowhere to be drawn".to_owned())?,
      )
    } else {
      None
    };
    if destination.iter().count() + beside.len() != self.formats.len() {
      return Err("The export frame has the wrong number of formats to draw".to_owned());
    }
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
    let camera = unsafe { pixel_buffer(frame.camera) }
      .map(|pixels| camera_source(gpu, pixels))
      .transpose()?;
    let destinations = destination.into_iter().chain(beside.iter().copied());
    let mut submission = None;
    for (format, destination) in self.formats.iter_mut().zip(destinations) {
      format.draw.draw(
        &format.compositor,
        &format.canvas,
        &source,
        camera.as_ref(),
        (frame.source_us, frame.frame_ms),
      )?;
      let luma = buffer_plane_texture(gpu, destination, 0, wgpu::TextureFormat::R8Unorm, "luma")?;
      let chroma =
        buffer_plane_texture(gpu, destination, 1, wgpu::TextureFormat::Rg8Unorm, "chroma")?;
      let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Screenwide export frame out"),
        });
      self.planes.encode(
        gpu,
        &mut encoder,
        &format.canvas,
        &view(luma),
        &view(chroma),
      );
      submission = Some(gpu.queue.submit([encoder.finish()]));
    }
    let submission = submission.ok_or_else(|| "The export has no format to draw".to_owned())?;
    let token = self.next_token;
    self.next_token += 1;
    self.submitted.push_back((token, submission));
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
    let source = super::frame_source::canvas_source(&self.gpu, size);
    self.source = Some(source.clone());
    source
  }
}

/// The decoded camera frame `pixels` as the canvas samples it.
fn camera_source(gpu: &Gpu, pixels: &cv::PixelBuf) -> Result<SourceTexture, String> {
  let texture = bgra_buffer_texture(gpu, pixels, "export camera")?;
  Ok(SourceTexture {
    size: (texture.width(), texture.height()),
    view: texture.create_view(&Default::default()),
    texture,
    picture: None,
    enhanced: false,
    cut_out: false,
  })
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
