// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shared device's Metal side: IOSurface-backed CoreVideo buffers and
//! textures other code owns as wgpu textures, without a copy, and the raw
//! device and queue for Objective-C that encodes beside wgpu.

use cidre::cv;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{
  MTLDevice, MTLPixelFormat, MTLTexture, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage,
};
use wgpu::hal::api::Metal;

use super::Gpu;

/// The shared device's `MTLDevice` and `MTLCommandQueue`, unretained. Command
/// buffers committed to this queue run after every wgpu submission made
/// before them, which is what lets Objective-C draw over a wgpu frame in the
/// same drawable. Both live as long as the shared device, for the process.
pub(crate) fn metal_device_and_queue(
  gpu: &Gpu,
) -> Result<(*mut std::ffi::c_void, *mut std::ffi::c_void), String> {
  let device = {
    let hal = unsafe { gpu.device.as_hal::<Metal>() }
      .ok_or_else(|| "The graphics device is not running on Metal".to_owned())?;
    Retained::as_ptr(hal.raw_device()).cast_mut().cast()
  };
  let queue = {
    let hal = unsafe { gpu.queue.as_hal::<Metal>() }
      .ok_or_else(|| "The graphics queue is not running on Metal".to_owned())?;
    std::ptr::from_ref(hal.as_raw()).cast_mut().cast()
  };
  Ok((device, queue))
}

/// The shared device's `id<MTLDevice>` for Objective-C that makes layers and
/// textures beside wgpu, or NULL when there is no device.
#[no_mangle]
pub extern "C" fn screenwide_shared_metal_device() -> *mut std::ffi::c_void {
  super::shared()
    .and_then(metal_device_and_queue)
    .map_or(std::ptr::null_mut(), |(device, _)| device)
}

/// The shared device's `id<MTLCommandQueue>`, on which a command buffer runs
/// after every wgpu submission made before it, or NULL when there is none.
#[no_mangle]
pub extern "C" fn screenwide_shared_metal_queue() -> *mut std::ffi::c_void {
  super::shared()
    .and_then(metal_device_and_queue)
    .map_or(std::ptr::null_mut(), |(_, queue)| queue)
}

/// `raw`, a BGRA `MTLTexture` on the shared device that someone else owns,
/// such as a `CAMetalDrawable`'s, as a texture wgpu renders into.
///
/// # Safety
/// `raw` must be a live `id<MTLTexture>` made by the shared device.
pub(crate) unsafe fn bgra_render_target(
  gpu: &Gpu,
  raw: *mut std::ffi::c_void,
) -> Result<wgpu::Texture, String> {
  let texture = unsafe {
    foreign_texture(
      gpu,
      raw,
      wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
    )
  }?;
  if texture.format() != wgpu::TextureFormat::Bgra8Unorm {
    return Err("The drawable is not BGRA".to_owned());
  }
  Ok(texture)
}

/// `raw`, an `MTLTexture` on the shared device that someone else owns, as a
/// texture wgpu samples: a text label or a frozen desktop Objective-C made.
///
/// # Safety
/// `raw` must be a live `id<MTLTexture>` made by the shared device.
pub(crate) unsafe fn sampled_texture(
  gpu: &Gpu,
  raw: *mut std::ffi::c_void,
) -> Result<wgpu::Texture, String> {
  unsafe { foreign_texture(gpu, raw, wgpu::TextureUsages::TEXTURE_BINDING) }
}

/// # Safety
/// `raw` must be a live `id<MTLTexture>` made by the shared device, whose
/// Metal usage covers `usage`.
unsafe fn foreign_texture(
  gpu: &Gpu,
  raw: *mut std::ffi::c_void,
  usage: wgpu::TextureUsages,
) -> Result<wgpu::Texture, String> {
  let raw = unsafe { Retained::retain(raw.cast::<ProtocolObject<dyn MTLTexture>>()) }
    .ok_or_else(|| "The Metal texture is missing".to_owned())?;
  let format = match raw.pixelFormat() {
    MTLPixelFormat::BGRA8Unorm => wgpu::TextureFormat::Bgra8Unorm,
    MTLPixelFormat::RGBA8Unorm => wgpu::TextureFormat::Rgba8Unorm,
    MTLPixelFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
    other => return Err(format!("A Metal texture in {other:?} cannot be shared")),
  };
  let (width, height) = (raw.width() as u32, raw.height() as u32);
  let hal = unsafe {
    wgpu::hal::metal::Device::texture_from_raw(
      raw,
      format,
      MTLTextureType::Type2D,
      1,
      1,
      wgpu::hal::CopyExtent {
        width,
        height,
        depth: 1,
      },
      None,
    )
  };
  Ok(unsafe {
    gpu.device.create_texture_from_hal::<Metal>(
      hal,
      &wgpu::TextureDescriptor {
        label: Some("Screenwide shared Metal texture"),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
      },
      wgpu::TextureUses::UNINITIALIZED,
    )
  })
}

/// `pixels`, a BGRA buffer backed by an IOSurface, as a texture wgpu samples
/// and renders into.
pub(crate) fn bgra_buffer_texture(
  gpu: &Gpu,
  pixels: &cv::PixelBuf,
  label: &str,
) -> Result<wgpu::Texture, String> {
  if pixels.pixel_format() != cv::PixelFormat::_32_BGRA {
    return Err(format!("The {label} is not BGRA"));
  }
  buffer_plane_texture(gpu, pixels, 0, wgpu::TextureFormat::Bgra8Unorm, label)
}

/// One plane of `pixels`, an IOSurface-backed buffer, as a texture wgpu
/// samples and renders into: `plane` 0 of a single-plane buffer is the whole
/// image. `format` has to match the plane's own layout, such as `R8Unorm`
/// for 4:2:0 luma and `Rg8Unorm` for its chroma. The texture retains the
/// IOSurface, so it may outlive the caller's hold on `pixels`.
pub(crate) fn buffer_plane_texture(
  gpu: &Gpu,
  pixels: &cv::PixelBuf,
  plane: usize,
  format: wgpu::TextureFormat,
  label: &str,
) -> Result<wgpu::Texture, String> {
  let metal_format = match format {
    wgpu::TextureFormat::Bgra8Unorm => MTLPixelFormat::BGRA8Unorm,
    wgpu::TextureFormat::R8Unorm => MTLPixelFormat::R8Unorm,
    wgpu::TextureFormat::Rg8Unorm => MTLPixelFormat::RG8Unorm,
    _ => return Err(format!("The {label} has no Metal format for {format:?}")),
  };
  let (width, height) = if pixels.plane_count() == 0 && plane == 0 {
    (pixels.width(), pixels.height())
  } else if plane < pixels.plane_count() {
    (pixels.plane_width(plane), pixels.plane_height(plane))
  } else {
    return Err(format!("The {label} has no plane {plane}"));
  };
  let (width, height) = (width as u32, height as u32);
  let surface = pixels
    .io_surf()
    .ok_or_else(|| format!("The {label} is not backed by an IOSurface"))?;
  // Both name the same CoreFoundation object.
  let surface = unsafe { &*std::ptr::from_ref(surface).cast::<IOSurfaceRef>() };
  let raw = {
    let hal = unsafe { gpu.device.as_hal::<Metal>() }
      .ok_or_else(|| "The graphics device is not running on Metal".to_owned())?;
    let descriptor = unsafe {
      MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
        metal_format,
        width as usize,
        height as usize,
        false,
      )
    };
    descriptor.setUsage(MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget);
    hal
      .raw_device()
      .newTextureWithDescriptor_iosurface_plane(&descriptor, surface, plane)
      .ok_or_else(|| format!("Metal could not wrap the {label}"))?
  };
  let hal = unsafe {
    wgpu::hal::metal::Device::texture_from_raw(
      raw,
      format,
      MTLTextureType::Type2D,
      1,
      1,
      wgpu::hal::CopyExtent {
        width,
        height,
        depth: 1,
      },
      None,
    )
  };
  Ok(unsafe {
    gpu.device.create_texture_from_hal::<Metal>(
      hal,
      &wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        // A redaction copies the frame before drawing over the copy; Metal
        // blits any texture, whatever usage it was made with.
        usage: wgpu::TextureUsages::TEXTURE_BINDING
          | wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      },
      wgpu::TextureUses::UNINITIALIZED,
    )
  })
}
