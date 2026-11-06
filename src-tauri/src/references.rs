// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Reference screenshots: a still kept on screen at its true size, where it
//! was taken, to compare what is there now against. Each is a borderless
//! window of its own, as many as the person takes, sized in points so it
//! keeps its size on a display of another scale. The picture is drawn on
//! the GPU through the OSC renderer, with the selection's handles to resize
//! it, in a native view over the window's webview, which draws the island
//! under it. They are throwaway: the picture's files are deleted when its
//! window closes, until Open in editor hands it to the screenshot editor.
//! The Ruler and Text Recognition read them, since their snapshots take
//! Screenwide's own windows; screenshots and recordings leave them out as
//! they do every Screenwide window (`app_windows::capture_affinity` on
//! Windows).

pub(crate) mod commands;
mod drawing;
mod drop;
pub(crate) mod drop_island;
#[cfg(target_os = "macos")]
mod drop_macos;
mod drop_watch;
#[cfg(target_os = "windows")]
mod drop_windows;
mod engine;
mod fit;
#[cfg(target_os = "macos")]
mod macos;
mod stage;
mod window;
#[cfg(target_os = "windows")]
mod windows;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use screenwide_annotations::Annotation;
use screenwide_imaging::CapturedImage;
use tauri::{AppHandle, Manager};

pub(crate) use drop_watch::start as start_drop_watch;
pub(crate) use window::{placement, Placement};

/// Every reference window's label starts so.
const LABEL_PREFIX: &str = "reference-";

/// A still to keep as a reference.
pub(crate) struct ReferencePicture {
  /// The still as taken, which the editor opens with `annotations` as a
  /// layer of their own.
  pub original: CapturedImage,
  /// What the window shows where live annotations were drawn over the still:
  /// `original` with them drawn in. None where there were none.
  pub shown: Option<CapturedImage>,
  pub annotations: Vec<Annotation>,
  /// How many of the still's pixels one point spans.
  pub scale: f64,
}

/// One reference on screen.
struct Reference {
  original: PathBuf,
  /// The same file as `original` where nothing was drawn in.
  shown: PathBuf,
  annotations: Vec<Annotation>,
  scale: f64,
  placement: Placement,
}

static REFERENCES: Mutex<Option<HashMap<String, Reference>>> = Mutex::new(None);
static NEXT: AtomicU64 = AtomicU64::new(1);

fn references() -> MutexGuard<'static, Option<HashMap<String, Reference>>> {
  REFERENCES.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether `label` is a reference's window, which Windows' capture policy
/// treats as its own kind.
#[cfg(target_os = "windows")]
pub(crate) fn is_reference_label(label: &str) -> bool {
  label.starts_with(LABEL_PREFIX)
}

/// Where the pictures live while their windows are open.
fn directory(app: &AppHandle) -> Result<PathBuf, String> {
  let directory = app
    .path()
    .temp_dir()
    .map_err(|error| error.to_string())?
    .join("Screenwide")
    .join("references");
  std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
  Ok(directory)
}

/// Clears pictures a previous run left behind, closed without its windows
/// getting to delete them.
pub(crate) fn clear_leftovers(app: &AppHandle) {
  if let Ok(directory) = directory(app) {
    let _ = std::fs::remove_dir_all(&directory);
  }
}

fn write_png(image: &CapturedImage, path: &PathBuf) -> Result<(), String> {
  use image::ImageEncoder;
  let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
  // Fast over small: the file is read back once, from this machine's disk.
  image::codecs::png::PngEncoder::new_with_quality(
    std::io::BufWriter::new(file),
    image::codecs::png::CompressionType::Fast,
    image::codecs::png::FilterType::Adaptive,
  )
  .write_image(
    &image.rgba,
    image.width,
    image.height,
    image::ExtendedColorType::Rgba8,
  )
  .map_err(|error| error.to_string())
}

fn read_png(path: &PathBuf) -> Result<CapturedImage, String> {
  let image = image::open(path)
    .map_err(|error| error.to_string())?
    .into_rgba8();
  Ok(CapturedImage {
    width: image.width(),
    height: image.height(),
    rgba: image.into_raw(),
  })
}

/// Keeps `picture` on screen as a reference at `placement`.
pub(crate) async fn open(
  app: &AppHandle,
  picture: ReferencePicture,
  placement: Placement,
) -> Result<(), String> {
  let label = format!("{LABEL_PREFIX}{}", NEXT.fetch_add(1, Ordering::Relaxed));
  let texture = upload(picture.shown.as_ref().unwrap_or(&picture.original))?;
  let directory = directory(app)?;
  let original = directory.join(format!("{label}.png"));
  let shown = match picture.shown {
    Some(_) => directory.join(format!("{label}-shown.png")),
    None => original.clone(),
  };
  let (writing, written_original, written_shown) = (picture.shown, original.clone(), shown.clone());
  let still = picture.original;
  tauri::async_runtime::spawn_blocking(move || {
    write_png(&still, &written_original)?;
    match writing {
      Some(drawn) => write_png(&drawn, &written_shown),
      None => Ok(()),
    }
  })
  .await
  .map_err(|error| error.to_string())??;
  references().get_or_insert_with(HashMap::new).insert(
    label.clone(),
    Reference {
      original,
      shown,
      annotations: picture.annotations,
      scale: picture.scale,
      placement,
    },
  );
  if let Err(error) = window::build(app, &label, placement, texture) {
    forget(&label);
    return Err(error);
  }
  Ok(())
}

/// Lets go of the reference `label`, its view and layers, and deletes its
/// pictures. Runs as its window is destroyed, on the main thread.
fn forget(label: &str) {
  engine::detach(label);
  let Some(reference) = references()
    .as_mut()
    .and_then(|references| references.remove(label))
  else {
    return;
  };
  let _ = std::fs::remove_file(&reference.original);
  if reference.shown != reference.original {
    let _ = std::fs::remove_file(&reference.shown);
  }
}

/// The still the reference shows, on the GPU, which draws it from there on.
/// A picture with see-through pixels keeps mip levels as well, which the
/// shadow it casts from what it shows is blurred from (`reference.wgsl`).
fn upload(image: &CapturedImage) -> Result<wgpu::Texture, String> {
  const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  let gpu = screenwide_gpu::shared()?;
  let see_through = image
    .rgba
    .as_chunks::<4>()
    .0
    .iter()
    .any(|pixel| pixel[3] < u8::MAX);
  if !see_through {
    return Ok(gpu.texture_with_pixels(
      "Screenwide reference",
      (image.width, image.height, 1),
      FORMAT,
      &image.rgba,
    ));
  }
  let size = wgpu::Extent3d {
    width: image.width,
    height: image.height,
    depth_or_array_layers: 1,
  };
  let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide see-through reference"),
    size,
    mip_level_count: screenwide_gpu::mips::levels((image.width, image.height)),
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::RENDER_ATTACHMENT
      | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  gpu.queue.write_texture(
    texture.as_image_copy(),
    &image.rgba,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(image.width * 4),
      rows_per_image: Some(image.height),
    },
    size,
  );
  static MIPS: OnceLock<screenwide_gpu::mips::Mips> = OnceLock::new();
  MIPS
    .get_or_init(|| screenwide_gpu::mips::Mips::new(&gpu.device, FORMAT))
    .fill(gpu, &texture);
  Ok(texture)
}
