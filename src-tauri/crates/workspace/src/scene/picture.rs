// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures layers are drawn from, and the one place where what a
//! platform's decoders hand over meets the shared scene.

use screenwide_compositor::compositor::{Compositor, SourceTexture};
use screenwide_imaging::CapturedImage;

#[cfg(target_os = "macos")]
use cidre::{arc, cv};

/// A Windows decoder's frame: its Direct3D 11 texture, the subresource that
/// holds the picture, and its size.
#[cfg(target_os = "windows")]
pub type DecodedTexture<'a> = (
  &'a windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
  u32,
  (u32, u32),
);

/// A picture a layer is drawn from: a decoded image; on macOS a CoreVideo
/// buffer a decoder vends, which is drawn from without a copy; on Windows a
/// frame a Media Foundation decoder holds, copied into the texture the scene
/// keeps for the layer; or, where the decoder had no new frame for the layer
/// this time, the picture the scene already holds for it.
#[derive(Clone, Copy)]
pub enum LayerPicture<'a> {
  Image(&'a CapturedImage),
  #[cfg(target_os = "macos")]
  Pixels(*mut std::ffi::c_void),
  #[cfg(target_os = "windows")]
  Decoded(DecodedTexture<'a>),
  Held,
}

/// A picture on the GPU, and on macOS the buffer a CoreVideo one is drawn
/// from: the scene holds the buffer, so its decoder cannot hand it to another
/// frame.
pub(super) struct ScenePicture {
  pub(super) texture: SourceTexture,
  /// Whether its pixels never change under it, so what is drawn from them
  /// can be kept. A texture decoded frames are copied into is refilled.
  pub(super) retained: bool,
  #[cfg(target_os = "macos")]
  _pixels: Option<arc::R<cv::PixelBuf>>,
}

// SAFETY: the buffer is only held to keep it alive and is never read through
// here; retaining and releasing a CVPixelBuffer is safe from any thread.
#[cfg(target_os = "macos")]
unsafe impl Send for ScenePicture {}
#[cfg(target_os = "macos")]
unsafe impl Sync for ScenePicture {}

impl ScenePicture {
  pub(super) fn new(compositor: &Compositor, picture: LayerPicture<'_>) -> Result<Self, String> {
    match picture {
      LayerPicture::Image(image) => Ok(Self {
        texture: compositor.screenshot_source(image)?,
        retained: true,
        #[cfg(target_os = "macos")]
        _pixels: None,
      }),
      #[cfg(target_os = "macos")]
      LayerPicture::Pixels(pointer) => {
        let pixels = unsafe { pointer.cast::<cv::PixelBuf>().as_ref() }
          .ok_or_else(|| "The workspace frame is missing".to_owned())?;
        let texture =
          screenwide_gpu::macos::bgra_buffer_texture(compositor.gpu(), pixels, "workspace frame")?;
        Ok(Self {
          retained: true,
          texture: SourceTexture {
            size: (texture.width(), texture.height()),
            view: texture.create_view(&Default::default()),
            texture,
            picture: None,
            enhanced: false,
            cut_out: false,
          },
          _pixels: Some(pixels.retained()),
        })
      }
      #[cfg(target_os = "windows")]
      LayerPicture::Decoded((frame, subresource, size)) => {
        let d3d11 = screenwide_gpu::d3d11()?;
        let texture = compositor.source(d3d11, size)?;
        compositor.copy_source(d3d11, &texture, frame, subresource)?;
        Ok(Self {
          texture,
          retained: false,
        })
      }
      LayerPicture::Held => Err("The workspace holds no picture for this layer yet".to_owned()),
    }
  }

  /// Whether this picture, the one the scene holds for a layer, can show
  /// `picture` too: as it is where nothing new arrived, or by having a
  /// decoded frame of its size copied into it. Every other picture is a new
  /// one.
  pub(super) fn takes(&self, picture: LayerPicture<'_>) -> bool {
    match picture {
      LayerPicture::Held => true,
      #[cfg(target_os = "windows")]
      LayerPicture::Decoded((_, _, size)) => self.texture.size == size,
      #[cfg(target_os = "macos")]
      LayerPicture::Pixels(_) => false,
      LayerPicture::Image(_) => false,
    }
  }

  /// Shows `picture`, which this picture `takes`.
  pub(super) fn refill(
    &self,
    compositor: &Compositor,
    picture: LayerPicture<'_>,
  ) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    if let LayerPicture::Decoded((frame, subresource, _)) = picture {
      compositor.copy_source(screenwide_gpu::d3d11()?, &self.texture, frame, subresource)?;
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (self, compositor, picture);
    Ok(())
  }
}

impl LayerPicture<'_> {
  pub(super) fn size(self) -> (u32, u32) {
    match self {
      Self::Image(image) => (image.width, image.height),
      #[cfg(target_os = "macos")]
      Self::Pixels(pointer) => unsafe { pointer.cast::<cv::PixelBuf>().as_ref() }
        .map_or((0, 0), |pixels| {
          (pixels.width() as u32, pixels.height() as u32)
        }),
      #[cfg(target_os = "windows")]
      Self::Decoded((_, _, size)) => size,
      Self::Held => (0, 0),
    }
  }

  /// Whether this is the picture the scene already holds for the layer.
  pub(super) fn is_held(self) -> bool {
    matches!(self, Self::Held)
  }
}
