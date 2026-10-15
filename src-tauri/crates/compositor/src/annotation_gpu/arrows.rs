// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The storage-buffer elements the annotation shader reads, and the buffers
//! that carry them. Layouts here are pinned against `annotations.wgsl` by
//! the asserts beside each struct, so a field added on one side fails the
//! build rather than drifting silently.

use super::FocusRegion;
use screenwide_annotations::geometry::{ArrowGeometry, ArrowTriangle};

/// One prepared arrow's geometry as scalars, shared by the annotation and by
/// each of its exposure samples. The twin of `PreviewGeometry` in
/// `annotations.wgsl`.
///
/// Every member is a scalar so the WGSL struct, also all scalars, packs at
/// four bytes with no alignment padding, exactly as this one does.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PreviewGeometry {
  pub(crate) ax: f32,
  pub(crate) ay: f32,
  pub(crate) bx: f32,
  pub(crate) by: f32,
  pub(crate) cx: f32,
  pub(crate) cy: f32,
  pub(crate) width: f32,
  pub(crate) low: f32,
  pub(crate) high: f32,
  pub(crate) start_head: [f32; 6],
  pub(crate) end_head: [f32; 6],
  pub(crate) rounding: f32,
  pub(crate) head: u32,
  /// For a caption piece that turns, flagged `TURNED`: the homography from
  /// the canvas back onto the piece as it lies unturned, row by row, whose
  /// third coordinate is positive only in front of the eye, and the canvas
  /// bounds the turned piece reaches, low then high. Zero for anything
  /// else, which never reads it.
  pub(crate) turn: [f32; 13],
}

const _: () = assert!(std::mem::size_of::<PreviewGeometry>() == 144);

impl PreviewGeometry {
  pub(crate) fn new(geometry: ArrowGeometry) -> Self {
    let triangle = |triangle: ArrowTriangle| {
      [
        triangle.a[0],
        triangle.a[1],
        triangle.b[0],
        triangle.b[1],
        triangle.c[0],
        triangle.c[1],
      ]
    };
    Self {
      ax: geometry.a[0],
      ay: geometry.a[1],
      bx: geometry.b[0],
      by: geometry.b[1],
      cx: geometry.c[0],
      cy: geometry.c[1],
      width: geometry.width,
      low: geometry.low,
      high: geometry.high,
      start_head: triangle(geometry.start_head),
      end_head: triangle(geometry.end_head),
      rounding: geometry.rounding,
      head: geometry.head,
      turn: [0.0; 13],
    }
  }
}

/// One prepared annotation as the shader's storage-buffer element. The field
/// order is the twin of `PreviewArrow` in `annotations.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PreviewArrow {
  pub(crate) geometry: PreviewGeometry,
  pub(crate) color: [f32; 4],
  pub(crate) hover: f32,
  pub(crate) sample_first: u32,
  pub(crate) sample_count: u32,
  pub(crate) kind: u32,
  pub(crate) flags: u32,
  pub(crate) params: [f32; 3],
  pub(crate) data_offset: u32,
  pub(crate) data_count: u32,
}

const _: () = assert!(std::mem::size_of::<PreviewArrow>() == 200);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, color) == 144);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, hover) == 160);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, sample_first) == 164);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, kind) == 172);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, flags) == 176);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, params) == 180);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, data_offset) == 192);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, data_count) == 196);

impl PreviewArrow {
  pub(crate) fn new(geometry: ArrowGeometry, color: [f32; 4], hover: f32, kind: u32) -> Self {
    Self {
      geometry: PreviewGeometry::new(geometry),
      color,
      hover,
      sample_first: 0,
      sample_count: 0,
      kind,
      flags: 0,
      params: [0.0; 3],
      data_offset: 0,
      data_count: 0,
    }
  }
}
/// One exposure sample: the annotation part way through the interval this frame
/// covers, and how solid it was then. The twin of `PreviewSample` in
/// `annotations.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PreviewSample {
  pub(crate) geometry: PreviewGeometry,
  pub(crate) opacity: f32,
}

const _: () = assert!(std::mem::size_of::<PreviewSample>() == 148);

impl PreviewSample {
  pub(crate) fn new(geometry: ArrowGeometry, opacity: f32) -> Self {
    Self {
      geometry: PreviewGeometry::new(geometry),
      opacity,
    }
  }
}

/// The most exposure samples one annotation prepares.
pub const MAX_EXPOSURE_SAMPLES: usize = 48;

/// One annotation's type as the atlas sets it: a counter's number, a text
/// box's lines, or nothing at all for an arrow.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreparedType {
  pub(crate) text: String,
  /// A counter's disc radius or a text box's type size, in canvas pixels.
  pub(crate) size: f32,
  /// Zero for a counter, one more than its alignment for a text box.
  pub(crate) style: u32,
  /// The caret and the selection of a box being typed into.
  pub(crate) marks: Option<screenwide_annotations::text::typing::TypingMarks>,
  /// The face the type is set in: Inter for annotations, the caption
  /// style's for captions.
  pub(crate) face: screenwide_annotations::text::face::TypeFace,
  /// How wide an outline is drawn round the type, in canvas pixels, which
  /// the atlas sets beside it; zero for none.
  pub(crate) outline: f32,
  /// How far the edge of the shadow a caption's word casts spreads, in
  /// canvas pixels, which the atlas blurs its outline by beside it; none
  /// for a word that casts none.
  pub(crate) shadow: Option<f32>,
}

/// One image a composition draws: its place in the list, its picture's
/// asset id, its full width and height in canvas pixels, which it is
/// rasterised at however far it has arrived, so a pop in draws it once, and
/// which frame of a moving picture this moment shows.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedImage {
  pub(crate) index: usize,
  pub(crate) asset: String,
  pub(crate) size: [f32; 2],
  /// The frame a still shows or playback starts on.
  pub(crate) frame: u32,
  /// How far into its clip this moment is, in milliseconds; none on a still.
  pub(crate) clock_ms: Option<f32>,
  /// Whether a moving picture plays through once rather than looping.
  pub(crate) once: bool,
}

/// Everything one composition draws for its annotations.
#[derive(Default)]
pub struct PreparedArrows {
  pub(crate) arrows: Vec<PreviewArrow>,
  /// Each annotation's type, in draw order.
  pub(crate) types: Vec<PreparedType>,
  /// The images, with where they are in `arrows`.
  pub(crate) images: Vec<PreparedImage>,
  pub points: Vec<[f32; 2]>,
  pub text: Vec<u8>,
  pub samples: Vec<PreviewSample>,
  /// Where the above-camera run starts, which is what the shader's two
  /// passes are bounded by.
  pub(crate) below_camera: u32,
  /// How many canvas pixels one drawn pixel covers. Every edge is feathered
  /// over this rather than over one canvas pixel: a canvas shown smaller
  /// than its resolution would otherwise take its whole antialiasing band
  /// from inside a single drawn pixel and come out jagged. Zero means one.
  pub pixel_scale: f32,
  /// The redactions the pre-pass applies to the source, in its own pixels.
  pub(crate) redactions: screenwide_annotations::redact::records::RedactRecords,
  /// The spotlights' blur the cursor and the marks under it take: its
  /// standard deviation in canvas pixels and how far it has arrived, both
  /// zero where no spotlight blurs.
  pub(crate) spotlight_blur: [f32; 2],
  /// The focus boxes this frame blurs around, in the source's own pixels;
  /// `compositor::focus` applies them.
  pub focus: Vec<FocusRegion>,
  /// Where in `arrows` the topmost focus sits: everything drawn before it
  /// is blurred, and the arrows from here on are drawn sharp over the blur.
  pub(crate) focus_cut: u32,
  /// Whether the topmost focus lies in the run above the camera, which
  /// decides, as it does for a spotlight's shade, whether the camera lies
  /// under it.
  pub(crate) focus_above_camera: bool,
}

impl PreparedArrows {
  /// Draws `above`, prepared in the same canvas pixels, over everything
  /// these draw, the camera included. Only text boxes are taken: their
  /// type follows in the side buffer, and they read no points.
  pub fn draw_above(&mut self, above: PreparedArrows) {
    let samples = self.samples.len() as u32;
    let text = self.text.len() as u32;
    self
      .arrows
      .extend(above.arrows.into_iter().map(|mut arrow| {
        arrow.sample_first += samples;
        arrow.data_offset += text;
        arrow
      }));
    self.types.extend(above.types);
    self.samples.extend(above.samples);
    self.text.extend(above.text);
    if self.pixel_scale <= 0.0 {
      self.pixel_scale = above.pixel_scale;
    }
  }
}

/// A CPU-written storage buffer that grows to fit the list it is handed. A
/// list is as long as its document, so nothing is sized up front: the buffer
/// is replaced by one twice the size needed whenever a list outgrows it, and
/// kept after that.
pub struct GpuBuffer {
  name: &'static str,
  slot: std::sync::Mutex<wgpu::Buffer>,
}

impl GpuBuffer {
  pub fn new(gpu: &screenwide_gpu::Gpu, name: &'static str) -> Self {
    const FIRST_BYTES: u64 = 4096;
    Self {
      name,
      slot: std::sync::Mutex::new(storage_buffer(gpu, FIRST_BYTES, name)),
    }
  }

  /// Writes `items` to the front of the buffer, growing it first when they do
  /// not fit, and returns the buffer to bind. Nothing is written for an empty
  /// list; the shader never reads past its counts.
  pub fn write<T: bytemuck::Pod>(
    &self,
    gpu: &screenwide_gpu::Gpu,
    items: &[T],
  ) -> Result<wgpu::Buffer, String> {
    let bytes: &[u8] = bytemuck::cast_slice(items);
    let mut slot = self
      .slot
      .lock()
      .map_err(|_| format!("The {} buffer is poisoned", self.name))?;
    // A storage write is a whole number of words; the text is bytes.
    let padded = (bytes.len() as u64).next_multiple_of(4);
    if padded > slot.size() {
      *slot = storage_buffer(gpu, padded.next_power_of_two(), self.name);
    }
    if bytes.len() as u64 == padded {
      gpu.queue.write_buffer(&slot, 0, bytes);
    } else {
      let mut words = bytes.to_vec();
      words.resize(padded as usize, 0);
      gpu.queue.write_buffer(&slot, 0, &words);
    }
    Ok(slot.clone())
  }
}

fn storage_buffer(gpu: &screenwide_gpu::Gpu, size: u64, name: &str) -> wgpu::Buffer {
  gpu.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some(name),
    size,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  })
}
