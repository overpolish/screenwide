// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction pre-pass. A redaction is applied to a copy of the source
//! rather than drawn over the canvas: the canvas, the crop ghost and the
//! magnifier all sample the source, and each of them must find the covered
//! pixels already gone. The source itself stays untouched, so a redaction
//! that moves, or is removed, uncovers exactly the pixels that were there.
//!
//! Each redaction draws over a fresh copy of what the last one left, so a
//! box drawn over another averages what the first left. Two targets take
//! turns being that copy.

use super::redact_pipeline::{bind_group_layout, draw, pipeline, records_buffer, RECORD_STRIDE};
use super::redact_targets::{target, Scratch, Target};
use super::*;
use crate::editor::annotations::redact::records::{
  RedactRecord, RedactRecords, REDACT_BLUR, REDACT_MOSAIC, REDACT_SPOTLIGHT,
};

const CELLS_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/redact_cells.wgsl"));
const ROWS_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/redact_rows.wgsl"));
const PAINT_SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/redact_paint.wgsl"));

/// The two copies redactions take turns drawing over, sized and formatted
/// like the source, and the last still they redacted.
struct Work {
  size: (u32, u32),
  format: wgpu::TextureFormat,
  copies: [Target; 2],
  /// The retained source the last pass redacted, what it redacted, and the
  /// copy it left the result in. A still redrawn for its chrome keeps it.
  kept: Option<(wgpu::Texture, RedactRecords, usize)>,
}

pub(crate) struct Redactor {
  layout: wgpu::BindGroupLayout,
  cells_pipeline: wgpu::RenderPipeline,
  rows_pipeline: wgpu::RenderPipeline,
  /// The paint pass for a video frame's BGRA copies and a screenshot's RGBA.
  paint_pipelines: [(wgpu::TextureFormat, wgpu::RenderPipeline); 2],
  records: std::sync::Mutex<wgpu::Buffer>,
  zones: GpuBuffer,
  /// The copies for each redaction slot.
  work: std::sync::Mutex<Vec<Option<Work>>>,
  /// The cells a classically pixelated box averages, one pixel a cell.
  cells: Scratch,
  /// A blurred box's rows, blurred along and premultiplied, one pixel a pixel
  /// of the box.
  rows: Scratch,
  /// Bound for the cells or rows a pass does not read.
  unused: wgpu::TextureView,
}

impl Redactor {
  pub(super) fn new(gpu: &Gpu) -> Self {
    let device = &gpu.device;
    let layout = bind_group_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Screenwide redaction layout"),
      bind_group_layouts: &[Some(&layout)],
      immediate_size: 0,
    });
    let pipeline = |source, format| pipeline(device, &pipeline_layout, source, format);
    let cells = Scratch::new(wgpu::TextureFormat::Rgba8Unorm);
    let rows = Scratch::new(wgpu::TextureFormat::Rgba16Float);
    let paint = |format| (format, pipeline(PAINT_SHADER, format));
    Self {
      cells_pipeline: pipeline(CELLS_SHADER, cells.format()),
      rows_pipeline: pipeline(ROWS_SHADER, rows.format()),
      paint_pipelines: [
        paint(wgpu::TextureFormat::Bgra8Unorm),
        paint(wgpu::TextureFormat::Rgba8Unorm),
      ],
      layout,
      records: std::sync::Mutex::new(records_buffer(gpu, 8)),
      zones: GpuBuffer::new(gpu, "Screenwide redaction zones"),
      work: std::sync::Mutex::new(Vec::new()),
      cells,
      rows,
      unused: gpu
        .texture_with_pixels(
          "Screenwide redaction placeholder",
          (1, 1, 1),
          wgpu::TextureFormat::Rgba8Unorm,
          &[0; 4],
        )
        .create_view(&Default::default()),
    }
  }

  /// Records the passes that redact `source` into `encoder`, and answers the
  /// view the canvas samples it through: a redacted copy where `redactions`
  /// cover anything, else `None` and the source's own. The copy is `slot`'s,
  /// which only the next pass in that slot draws over. `retained` says the
  /// source's pixels never change under it, which lets a source redrawn with
  /// the same redactions reuse the copy it left.
  pub(super) fn apply(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    source: &SourceTexture,
    redactions: &RedactRecords,
    slot: usize,
    retained: bool,
  ) -> Result<Option<wgpu::TextureView>, String> {
    if redactions.records.is_empty() {
      return Ok(None);
    }
    let format = source.texture.format();
    let paint = &self
      .paint_pipelines
      .iter()
      .find(|(held, _)| *held == format)
      .ok_or_else(|| format!("A {format:?} source cannot be redacted"))?
      .1;
    let size = source.size;
    let mut slots = self
      .work
      .lock()
      .map_err(|_| "The redaction copies are poisoned".to_owned())?;
    if slots.len() <= slot {
      slots.resize_with(slot + 1, || None);
    }
    let work = &mut slots[slot];
    if work
      .as_ref()
      .is_none_or(|work| work.size != size || work.format != format)
    {
      *work = Some(Work {
        size,
        format,
        copies: [target(gpu, size, format), target(gpu, size, format)],
        kept: None,
      });
    }
    let work = work.as_mut().expect("the redaction copies were just made");
    if let Some((kept, records, index)) = &work.kept {
      if retained && *kept == source.texture && records == redactions {
        return Ok(Some(work.copies[*index].view.clone()));
      }
    }
    // Whatever is kept is about to be drawn over.
    work.kept = None;
    let records = self.write_records(gpu, &redactions.records)?;
    let zones = self.zones.write(gpu, &redactions.zones)?;
    let mut last = (&source.texture, &source.view);
    let mut index = 0;
    for (pass, record) in redactions.records.iter().enumerate() {
      index = pass % 2;
      let copy = &work.copies[index];
      encoder.copy_texture_to_texture(
        last.0.as_image_copy(),
        copy.texture.as_image_copy(),
        copy.texture.size(),
      );
      let offset = (pass as u64 * RECORD_STRIDE) as u32;
      let [cells, rows] = self.prepare(gpu, encoder, record, &records, offset, &zones, last.1)?;
      let bindings = self.bind_group(gpu, &records, last.1, &zones, &cells, &rows);
      let [x0, y0, x1, y1] = record.bounds;
      draw(
        encoder,
        &copy.view,
        [x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32],
        paint,
        &bindings,
        offset,
      );
      last = (&copy.texture, &copy.view);
    }
    if retained {
      work.kept = Some((source.texture.clone(), redactions.clone(), index));
    }
    Ok(Some(last.1.clone()))
  }

  /// Records the pass `record` needs before it is painted - a classic
  /// pixelation's cells or a blur's rows - from `from`, and answers the cells
  /// and rows the paint pass reads.
  #[allow(clippy::too_many_arguments)]
  fn prepare(
    &self,
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    record: &RedactRecord,
    records: &wgpu::Buffer,
    offset: u32,
    zones: &wgpu::Buffer,
    from: &wgpu::TextureView,
  ) -> Result<[wgpu::TextureView; 2], String> {
    let [x0, y0, x1, y1] = record.bounds;
    let (scratch, size, pipeline) = match record.mode {
      REDACT_MOSAIC => (
        &self.cells,
        (record.grid[0], record.grid[1]),
        &self.cells_pipeline,
      ),
      REDACT_BLUR | REDACT_SPOTLIGHT => (&self.rows, (x1 - x0, y1 - y0), &self.rows_pipeline),
      _ => return Ok([self.unused.clone(), self.unused.clone()]),
    };
    let view = scratch.view(gpu, size)?;
    let bindings = self.bind_group(gpu, records, from, zones, &self.unused, &self.unused);
    draw(
      encoder,
      &view,
      [0.0, 0.0, size.0 as f32, size.1 as f32],
      pipeline,
      &bindings,
      offset,
    );
    Ok(if record.mode == REDACT_MOSAIC {
      [view, self.unused.clone()]
    } else {
      [self.unused.clone(), view]
    })
  }

  /// Writes every record at its own stride, growing the buffer first when
  /// they do not fit.
  fn write_records(&self, gpu: &Gpu, records: &[RedactRecord]) -> Result<wgpu::Buffer, String> {
    let mut buffer = self
      .records
      .lock()
      .map_err(|_| "The redaction records are poisoned".to_owned())?;
    let needed = records.len() as u64 * RECORD_STRIDE;
    if needed > buffer.size() {
      *buffer = records_buffer(gpu, records.len().next_power_of_two() as u64);
    }
    let mut bytes = vec![0_u8; needed as usize];
    for (index, record) in records.iter().enumerate() {
      let start = index * RECORD_STRIDE as usize;
      bytes[start..start + size_of::<RedactRecord>()].copy_from_slice(bytemuck::bytes_of(record));
    }
    gpu.queue.write_buffer(&buffer, 0, &bytes);
    Ok(buffer.clone())
  }

  fn bind_group(
    &self,
    gpu: &Gpu,
    records: &wgpu::Buffer,
    source: &wgpu::TextureView,
    zones: &wgpu::Buffer,
    cells: &wgpu::TextureView,
    rows: &wgpu::TextureView,
  ) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Screenwide redaction bindings"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: records,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<RedactRecord>() as u64),
          }),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(source),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: zones.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: wgpu::BindingResource::TextureView(cells),
        },
        wgpu::BindGroupEntry {
          binding: 4,
          resource: wgpu::BindingResource::TextureView(rows),
        },
      ],
    })
  }
}
