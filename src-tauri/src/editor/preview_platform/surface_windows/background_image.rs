// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas's own background picture, uploaded once and sampled by the
//! preview shader. The CPU compose path fills the picture to the canvas with
//! `screenshots::background_image_canvas`; here the picture keeps its own size
//! on the GPU and the shader does the cover fit, so one upload serves every
//! canvas size, every preview frame, and every exported frame.
//!
//! Nothing here reports an error. A picture that cannot be read or decoded
//! answers `None`, the caller leaves the shader's flag clear, and the solid
//! colour paints instead: a background whose file has moved must never stop a
//! preview or an export.

use std::{
  collections::HashMap,
  ffi::c_void,
  sync::{Arc, Mutex},
  time::SystemTime,
};

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
      D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
      D3D11_USAGE_IMMUTABLE,
    },
    Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

/// Backgrounds kept resident at once. A workspace composes one picture per
/// layer, so a handful covers a whole compose pass without letting the cache
/// grow with every picture a session ever chose.
const CACHE_ENTRIES: usize = 4;

pub(super) struct BackgroundImage {
  _texture: ID3D11Texture2D,
  pub(super) view: ID3D11ShaderResourceView,
}

/// What the cached upload was made from. A picture edited in place keeps its
/// path, so the file's modification time and length are part of the identity.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Stamp {
  length: u64,
  modified: Option<SystemTime>,
}

struct Entry {
  image: Arc<BackgroundImage>,
  stamp: Stamp,
}

#[derive(Default)]
pub(super) struct BackgroundImageCache {
  entries: Mutex<HashMap<String, Entry>>,
}

fn stamp(path: &str) -> Stamp {
  let metadata = std::fs::metadata(path).ok();
  Stamp {
    length: metadata.as_ref().map_or(0, std::fs::Metadata::len),
    modified: metadata.and_then(|metadata| metadata.modified().ok()),
  }
}

fn upload(device: &ID3D11Device, path: &str) -> Option<BackgroundImage> {
  let pixels = image::open(path).ok()?.into_rgba8();
  let (width, height) = pixels.dimensions();
  if width == 0 || height == 0 {
    return None;
  }
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_IMMUTABLE,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let data = D3D11_SUBRESOURCE_DATA {
    pSysMem: pixels.as_raw().as_ptr().cast::<c_void>(),
    SysMemPitch: width * 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&data), Some(&mut texture)) }.ok()?;
  let texture = texture?;
  let resource: ID3D11Resource = texture.cast().ok()?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }.ok()?;
  Some(BackgroundImage {
    _texture: texture,
    view: view?,
  })
}

impl BackgroundImageCache {
  /// The uploaded picture behind this canvas, decoded at its own size. The
  /// upload is reused until the path changes or the file behind it does.
  pub(super) fn resolve(&self, device: &ID3D11Device, path: &str) -> Option<Arc<BackgroundImage>> {
    let stamp = stamp(path);
    let mut entries = self.entries.lock().ok()?;
    if let Some(entry) = entries.get(path) {
      if entry.stamp == stamp {
        return Some(Arc::clone(&entry.image));
      }
    }
    let image = Arc::new(upload(device, path)?);
    if entries.len() >= CACHE_ENTRIES {
      entries.clear();
    }
    entries.insert(
      path.to_owned(),
      Entry {
        image: Arc::clone(&image),
        stamp,
      },
    );
    Some(image)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_picture_that_is_not_there_has_an_empty_stamp() {
    assert_eq!(
      stamp("/no/such/background.png"),
      Stamp {
        length: 0,
        modified: None,
      }
    );
  }
}
