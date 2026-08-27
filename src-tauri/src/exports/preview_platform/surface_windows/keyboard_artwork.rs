// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! CPU rasterisation and D3D11 upload of the keyboard-shortcut artwork strip.
//! Mirrors the macOS Core Graphics rasteriser so both backends feed the same
//! shader geometry: one 20pt-tall strip of rounded key caps, drawn once per
//! appearance/density/shortcut and animated entirely on the GPU afterwards.

use std::{collections::HashMap, ffi::c_void, sync::Mutex};

use windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
        D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_IMMUTABLE,
      },
      Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      Gdi::{
        CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject,
        GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextCharacterExtra, SetTextColor,
        TextOutW, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_SWISS, FW_NORMAL, OUT_DEFAULT_PRECIS, TRANSPARENT,
        VARIABLE_PITCH,
      },
    },
  },
};

use crate::exports::keyboard_effects::{KeyboardKey, KeyboardOverlay};

#[path = "keyboard_artwork/visible_bounds.rs"]
mod visible_bounds;

/// Must match the `keyboard_key_*` array lengths in `preview.hlsl`.
const MAX_KEYS: usize = 8;
/// The React Keyboard's default variant: Inter text-sm/5 with tracking-wider
/// inside a `px-1 rounded-sm` cap on a 20pt line box.
const DESIGN_HEIGHT: f64 = 20.0;
const DESIGN_INSET: f64 = 4.0;
const DESIGN_GAP: f64 = 4.0;
const DESIGN_RADIUS: f64 = 4.0;
const FONT_SIZE: f64 = 14.0;
const FONT_KERN: f64 = 0.7;
const CACHE_ENTRIES: usize = 64;
const CACHE_BYTES: usize = 64 * 1024 * 1024;

/// The `Keyboard` constant buffer (b1) of `preview.hlsl`. HLSL pads each
/// element of a struct array to 16 bytes, so the per-key fields are stored as
/// parallel `uint4`/`float4` arrays rather than as one struct array.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct KeyboardConstants {
  pub(super) dimensions: [u32; 4],
  pub(super) animation: [f32; 4],
  pub(super) position: [f32; 4],
  pub(super) key_geometry: [[u32; 4]; MAX_KEYS],
  pub(super) key_motion: [[f32; 4]; MAX_KEYS],
  pub(super) key_masks: [[u32; 4]; MAX_KEYS],
  pub(super) key_position: [[f32; 4]; MAX_KEYS],
}

const _: () = assert!(size_of::<KeyboardConstants>() == 560);

impl Default for KeyboardConstants {
  fn default() -> Self {
    Self {
      dimensions: [0; 4],
      animation: [0.0; 4],
      position: [0.0; 4],
      key_geometry: [[0; 4]; MAX_KEYS],
      key_motion: [[0.0; 4]; MAX_KEYS],
      key_masks: [[0; 4]; MAX_KEYS],
      key_position: [[-1.0, -1.0, 1.0, 0.0]; MAX_KEYS],
    }
  }
}

/// One rasterised artwork strip: premultiplied BGRA pixels plus the artwork-space
/// span of every key cap.
pub(super) struct KeyboardRaster {
  pub(super) pixels: Vec<u8>,
  pub(super) size: (u32, u32),
  /// Artwork-space `(x, width)` of every key, in the prepared order.
  pub(super) keys: Vec<(u32, u32)>,
}

pub(super) struct KeyboardArtwork {
  /// Kept alive for the view; never read back.
  _texture: ID3D11Texture2D,
  bytes: usize,
  keys: Vec<(u32, u32)>,
  size: (u32, u32),
  pub(super) view: ID3D11ShaderResourceView,
}

#[derive(Default)]
pub(super) struct KeyboardArtworkCache {
  entries: Mutex<HashMap<String, std::sync::Arc<KeyboardArtwork>>>,
}

/// Windows virtual keys are normalised to macOS virtual keycodes at capture
/// time, so this table matches the macOS rasteriser except for the modifier
/// names, which follow the Windows keyboard.
fn key_label(code: u16) -> String {
  let label = match code {
    0 => "A",
    1 => "S",
    2 => "D",
    3 => "F",
    4 => "H",
    5 => "G",
    6 => "Z",
    7 => "X",
    8 => "C",
    9 => "V",
    11 => "B",
    12 => "Q",
    13 => "W",
    14 => "E",
    15 => "R",
    16 => "Y",
    17 => "T",
    18 => "1",
    19 => "2",
    20 => "3",
    21 => "4",
    22 => "6",
    23 => "5",
    24 => "=",
    25 => "9",
    26 => "7",
    27 => "\u{2212}",
    28 => "8",
    29 => "0",
    30 => "]",
    31 => "O",
    32 => "U",
    33 => "[",
    34 => "I",
    35 => "P",
    36 => "Enter",
    37 => "L",
    38 => "J",
    39 => "'",
    40 => "K",
    41 => ";",
    42 => "\\",
    43 => ",",
    44 => "/",
    45 => "N",
    46 => "M",
    47 => ".",
    48 => "Tab",
    49 => "Space",
    50 => "`",
    51 => "Backspace",
    53 => "Esc",
    54 | 55 => "Win",
    56 | 60 => "Shift",
    57 => "Caps Lock",
    58 | 61 => "Alt",
    59 | 62 => "Ctrl",
    63 => "fn",
    65 => ".",
    67 => "*",
    69 => "+",
    71 => "Num Lock",
    75 => "/",
    76 => "Enter",
    78 => "\u{2212}",
    81 => "=",
    82 => "0",
    83 => "1",
    84 => "2",
    85 => "3",
    86 => "4",
    87 => "5",
    88 => "6",
    89 => "7",
    91 => "8",
    92 => "9",
    96 => "F5",
    97 => "F6",
    98 => "F7",
    99 => "F3",
    100 => "F8",
    101 => "F9",
    103 => "F11",
    105 => "F13",
    106 => "F16",
    107 => "F14",
    109 => "F10",
    111 => "F12",
    113 => "F15",
    114 => "Insert",
    115 => "Home",
    116 => "Page Up",
    117 => "Del",
    118 => "F4",
    119 => "End",
    120 => "F2",
    121 => "Page Down",
    122 => "F1",
    123 => "\u{2190}",
    124 => "\u{2192}",
    125 => "\u{2193}",
    126 => "\u{2191}",
    // Passthrough keys with no macOS position, 0x0200 | the virtual key.
    0x0213 => "Pause",
    0x022c => "PrtScn",
    0x025d => "Menu",
    0x0291 => "Scroll Lock",
    _ => return format!("Key {code}"),
  };
  label.to_owned()
}

fn is_modifier_key(code: u16) -> bool {
  matches!(code, 54..=56 | 58..=63)
}

/// Version-one sidecars stored a modifier mask on the single recorded key.
/// Expanding it here keeps old recordings looking like the grouped shortcut.
fn prepared_shortcut(overlay: &KeyboardOverlay) -> Vec<(u16, KeyboardKey)> {
  let mut prepared = Vec::with_capacity(MAX_KEYS);
  let count = (overlay.key_count as usize).min(MAX_KEYS);
  for state in overlay.keys.iter().take(count) {
    if count == 1 && !is_modifier_key(state.key_code) {
      for (bit, code) in [55_u16, 59, 58, 56, 63].into_iter().enumerate() {
        if state.modifier_mask & (1 << bit) != 0 && prepared.len() < MAX_KEYS {
          prepared.push((code, *state));
        }
      }
    }
    if prepared.len() < MAX_KEYS {
      prepared.push((state.key_code, *state));
    }
  }
  prepared
}

/// Density the strip is rasterised at. The pop spring peaks just below 1.073,
/// so covering 1.08 guarantees animation never enlarges artwork past its own
/// source pixels.
pub(super) fn keyboard_backing_scale(output_height: u32, overlay: &KeyboardOverlay) -> f64 {
  const MAXIMUM_ANIMATED_SCALE: f64 = 1.08;
  let requested = if overlay.requested_scale > 0.0 {
    overlay.requested_scale
  } else {
    overlay.scale
  };
  let pixels = f64::from(output_height)
    * (60.0 / 1080.0)
    * f64::from(requested).max(0.0)
    * MAXIMUM_ANIMATED_SCALE;
  (pixels / DESIGN_HEIGHT).ceil().clamp(12.0, 64.0)
}

fn rounded_coverage(x: f64, y: f64, left: f64, width: f64, height: f64, radius: f64) -> f64 {
  let half_width = width * 0.5;
  let half_height = height * 0.5;
  let radius = radius.min(half_width).min(half_height).max(0.0);
  let local_x = (x - (left + half_width)).abs() - (half_width - radius);
  let local_y = (y - half_height).abs() - (half_height - radius);
  let outside = local_x.max(0.0).hypot(local_y.max(0.0));
  let distance = outside + local_x.max(local_y).min(0.0) - radius;
  (0.5 - distance).clamp(0.0, 1.0)
}

struct TextDevice {
  dc: windows::Win32::Graphics::Gdi::HDC,
  font: windows::Win32::Graphics::Gdi::HFONT,
  old_font: windows::Win32::Graphics::Gdi::HGDIOBJ,
}

impl Drop for TextDevice {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.dc, self.old_font);
      let _ = DeleteObject(self.font.into());
      let _ = DeleteDC(self.dc);
    }
  }
}

impl TextDevice {
  fn new(backing_scale: f64) -> Result<Self, String> {
    super::selection::label::register_inter_font();
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
      return Err("Windows could not create a keyboard artwork drawing context".to_owned());
    }
    let face: Vec<u16> = "Inter\0".encode_utf16().collect();
    let font = unsafe {
      CreateFontW(
        -(((FONT_SIZE * backing_scale).round() as i32).max(1)),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        ANTIALIASED_QUALITY,
        (VARIABLE_PITCH.0 | FF_SWISS.0) as u32,
        PCWSTR(face.as_ptr()),
      )
    };
    if font.is_invalid() {
      let _ = unsafe { DeleteDC(dc) };
      return Err("Windows could not create the keyboard artwork font".to_owned());
    }
    let old_font = unsafe { SelectObject(dc, font.into()) };
    // GDI has no per-run kerning attribute; intercharacter spacing is the
    // equivalent of AppKit's `NSKernAttributeName` and is applied to both
    // measurement and drawing, so the two stay consistent.
    unsafe { SetTextCharacterExtra(dc, (FONT_KERN * backing_scale).round() as i32) };
    Ok(Self { dc, font, old_font })
  }

  fn measure(&self, text: &[u16]) -> Result<(i32, i32), String> {
    let mut extent = Default::default();
    let measured = unsafe { GetTextExtentPoint32W(self.dc, text, &mut extent) };
    if !measured.as_bool() || extent.cx <= 0 || extent.cy <= 0 {
      return Err("Windows could not measure a keyboard shortcut label".to_owned());
    }
    Ok((extent.cx, extent.cy))
  }
}

/// Rasterises the shortcut strip at `backing_scale` device pixels per design
/// point. Needs no D3D device, so the geometry it produces is unit-testable.
pub(super) fn rasterize_keyboard(
  labels: &[String],
  light: bool,
  backing_scale: f64,
) -> Result<KeyboardRaster, String> {
  if labels.is_empty() {
    return Err("The keyboard shortcut has no keys to draw".to_owned());
  }
  let device = TextDevice::new(backing_scale)?;
  let wide: Vec<Vec<u16>> = labels
    .iter()
    .map(|label| label.encode_utf16().collect())
    .collect();
  let mut measured = Vec::with_capacity(labels.len());
  for text in &wide {
    measured.push(device.measure(text)?);
  }
  // Widths are decided in design points exactly as on macOS so the fitted
  // maximum width from the shared geometry table stays meaningful.
  let text_widths: Vec<f64> = measured
    .iter()
    .map(|(cx, _)| f64::from(*cx) / backing_scale)
    .collect();
  let key_widths: Vec<f64> = text_widths
    .iter()
    .map(|width| width.ceil() + DESIGN_INSET * 2.0)
    .collect();
  let design_width =
    key_widths.iter().sum::<f64>() + DESIGN_GAP * (key_widths.len().saturating_sub(1)) as f64;
  let width = ((design_width * backing_scale).ceil() as u32).max(1);
  let height = ((DESIGN_HEIGHT * backing_scale).ceil() as u32).max(1);

  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width as i32,
      biHeight: -(height as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };
  let mut bits = std::ptr::null_mut();
  let bitmap = unsafe {
    CreateDIBSection(
      Some(device.dc),
      &raw const info,
      DIB_RGB_COLORS,
      &mut bits,
      None,
      0,
    )
  }
  .map_err(|error| error.to_string())?;
  let old_bitmap = unsafe { SelectObject(device.dc, bitmap.into()) };
  let length = (width as usize) * (height as usize) * 4;
  let raster = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), length) };
  // Black, opaque ground: GDI blends white glyphs into it, so the red channel
  // of the result is plain coverage.
  for pixel in raster.chunks_exact_mut(4) {
    pixel.fill(0);
    pixel[3] = 255;
  }
  let mut key_x = 0.0_f64;
  let mut drawn = true;
  unsafe {
    SetBkMode(device.dc, TRANSPARENT);
    SetTextColor(device.dc, COLORREF(0x00FF_FFFF));
  }
  let mut keys = Vec::with_capacity(labels.len());
  for (index, text) in wide.iter().enumerate() {
    let key_width = key_widths[index];
    let text_x = key_x + (key_width - text_widths[index]) * 0.5;
    let y = (height as i32 - measured[index].1) / 2;
    drawn &=
      unsafe { TextOutW(device.dc, (text_x * backing_scale).round() as i32, y, text) }.as_bool();
    keys.push((
      (key_x * backing_scale).round() as u32,
      (key_width * backing_scale).round() as u32,
    ));
    key_x += key_width + DESIGN_GAP;
  }
  let coverage: Vec<u8> = raster.chunks_exact(4).map(|pixel| pixel[2]).collect();
  unsafe {
    SelectObject(device.dc, old_bitmap);
    let _ = DeleteObject(bitmap.into());
  }
  if !drawn {
    return Err("Windows could not draw a keyboard shortcut label".to_owned());
  }

  let (background, foreground) = if light { (229.0, 64.0) } else { (64.0, 163.0) };
  let radius = DESIGN_RADIUS * backing_scale;
  let mut pixels = vec![0_u8; length];
  for (cap_x, cap_pixels) in &keys {
    let left = f64::from(*cap_x);
    let cap_width = f64::from(*cap_pixels);
    let first = cap_x.saturating_sub(2);
    let last = (cap_x + cap_pixels + 2).min(width);
    for row in 0..height {
      let y = f64::from(row) + 0.5;
      for column in first..last {
        let cap = rounded_coverage(
          f64::from(column) + 0.5,
          y,
          left,
          cap_width,
          f64::from(height),
          radius,
        );
        if cap <= 0.0 {
          continue;
        }
        let offset = ((row as usize) * (width as usize) + column as usize) * 4;
        let value = (background * cap).round().clamp(0.0, 255.0) as u8;
        pixels[offset] = value;
        pixels[offset + 1] = value;
        pixels[offset + 2] = value;
        pixels[offset + 3] = (cap * 255.0).round().clamp(0.0, 255.0) as u8;
      }
    }
  }
  for (pixel, text) in pixels.chunks_exact_mut(4).zip(coverage) {
    if text == 0 {
      continue;
    }
    let text = f64::from(text) / 255.0;
    let keep = 1.0 - text;
    let alpha = text + (f64::from(pixel[3]) / 255.0) * keep;
    for channel in pixel[..3].iter_mut() {
      let value = foreground * text + f64::from(*channel) * keep;
      *channel = value.round().clamp(0.0, 255.0) as u8;
    }
    pixel[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
  }
  Ok(KeyboardRaster {
    pixels,
    size: (width, height),
    keys,
  })
}

/// Copies the animation state of every prepared key into the shader uniforms.
fn update_uniforms(
  values: &mut KeyboardConstants,
  overlay: &KeyboardOverlay,
  prepared: &[(u16, KeyboardKey)],
) {
  values.dimensions[2] = prepared.len() as u32;
  values.dimensions[3] = overlay.animation;
  values.animation = [
    overlay.scale,
    overlay.progress,
    overlay.maximum_width,
    overlay.requested_scale,
  ];
  values.position = [overlay.center_x, overlay.center_y, 0.0, 0.0];
  for (index, (_, state)) in prepared.iter().enumerate() {
    values.key_geometry[index][2] = state.visible;
    values.key_geometry[index][3] = state.slot;
    values.key_motion[index] = [
      state.alpha,
      state.scale,
      state.progress,
      state.layout_progress,
    ];
    values.key_masks[index] = [state.layout_from_mask, state.layout_to_mask, 0, 0];
    values.key_position[index] = [state.center_x, state.center_y, state.scale_ratio, 0.0];
  }
}

fn upload(device: &ID3D11Device, raster: &KeyboardRaster) -> Result<KeyboardArtwork, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: raster.size.0,
    Height: raster.size.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_IMMUTABLE,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let initial = D3D11_SUBRESOURCE_DATA {
    pSysMem: raster.pixels.as_ptr().cast::<c_void>(),
    SysMemPitch: raster.size.0 * 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no keyboard artwork texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(KeyboardArtwork {
    _texture: texture,
    bytes: raster.pixels.len(),
    keys: raster.keys.clone(),
    size: raster.size,
    view: view.ok_or_else(|| "D3D11 created no keyboard artwork view".to_owned())?,
  })
}

impl KeyboardArtworkCache {
  pub(super) fn visible_bounds(
    &self,
    device: &ID3D11Device,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    Ok(
      self
        .resolve(device, overlay, output.1)?
        .and_then(|(_, values)| visible_bounds::calculate(&values, output)),
    )
  }

  /// Returns the artwork strip for `overlay` and the shader uniforms that
  /// place it, rasterising and uploading only when the appearance, density or
  /// shortcut changed.
  pub(super) fn resolve(
    &self,
    device: &ID3D11Device,
    overlay: &KeyboardOverlay,
    output_height: u32,
  ) -> Result<Option<(std::sync::Arc<KeyboardArtwork>, KeyboardConstants)>, String> {
    if overlay.key_count == 0 {
      return Ok(None);
    }
    let prepared = prepared_shortcut(overlay);
    if prepared.is_empty() {
      return Ok(None);
    }
    let backing_scale = keyboard_backing_scale(output_height, overlay);
    let mut cache_key = format!("{}|{backing_scale:.0}|", overlay.appearance);
    for (code, _) in &prepared {
      cache_key.push_str(&format!("{code}:"));
    }
    let mut entries = self
      .entries
      .lock()
      .map_err(|_| "The keyboard artwork cache is unavailable".to_owned())?;
    let artwork = match entries.get(&cache_key) {
      Some(artwork) => std::sync::Arc::clone(artwork),
      None => {
        let labels = prepared
          .iter()
          .map(|(code, _)| key_label(*code))
          .collect::<Vec<_>>();
        let raster = rasterize_keyboard(
          &labels,
          overlay.appearance == KeyboardOverlay::APPEARANCE_LIGHT,
          backing_scale,
        )?;
        let artwork = std::sync::Arc::new(upload(device, &raster)?);
        let cached_bytes = entries.values().map(|entry| entry.bytes).sum::<usize>();
        if entries.len() >= CACHE_ENTRIES || cached_bytes + artwork.bytes > CACHE_BYTES {
          entries.clear();
        }
        entries.insert(cache_key, std::sync::Arc::clone(&artwork));
        artwork
      }
    };
    let mut values = KeyboardConstants {
      dimensions: [artwork.size.0, artwork.size.1, 0, 0],
      ..Default::default()
    };
    for (index, (x, width)) in artwork.keys.iter().enumerate().take(prepared.len()) {
      values.key_geometry[index][0] = *x;
      values.key_geometry[index][1] = *width;
    }
    update_uniforms(&mut values, overlay, &prepared);
    Ok(Some((artwork, values)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn overlay_with(codes: &[u16]) -> KeyboardOverlay {
    let mut overlay = KeyboardOverlay {
      key_count: codes.len() as u32,
      ..Default::default()
    };
    for (index, code) in codes.iter().enumerate() {
      overlay.keys[index] = KeyboardKey {
        key_code: *code,
        visible: 1,
        alpha: 1.0,
        scale: 1.0,
        progress: 1.0,
        slot: index as u32,
        ..Default::default()
      };
    }
    overlay
  }

  #[test]
  fn keyboard_constants_match_the_shader_register_packing() {
    assert_eq!(size_of::<KeyboardConstants>(), 560);
    assert_eq!(size_of::<KeyboardConstants>() % 16, 0);
  }

  #[test]
  fn key_caps_advance_by_a_four_point_gap_at_the_backing_scale() {
    let labels = ["Ctrl".to_owned(), "Shift".to_owned(), "P".to_owned()];
    let raster = rasterize_keyboard(&labels, true, 12.0).unwrap();

    assert_eq!(raster.keys.len(), labels.len());
    assert_eq!(raster.keys[0].0, 0);
    for key in &raster.keys {
      assert!(key.1 > 0, "every key cap has a positive width");
    }
    for pair in raster.keys.windows(2) {
      let gap = i64::from(pair[1].0) - i64::from(pair[0].0 + pair[0].1);
      assert_eq!(gap, (DESIGN_GAP * 12.0) as i64);
    }
    let last = raster.keys.last().copied().unwrap();
    assert!(last.0 + last.1 <= raster.size.0);
    assert_eq!(raster.size.1, (DESIGN_HEIGHT * 12.0) as u32);
  }

  #[test]
  fn artwork_pixels_stay_premultiplied() {
    let raster = rasterize_keyboard(&["Esc".to_owned()], false, 12.0).unwrap();

    assert_eq!(
      raster.pixels.len(),
      raster.size.0 as usize * raster.size.1 as usize * 4
    );
    assert!(raster
      .pixels
      .chunks_exact(4)
      .all(|pixel| pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]));
    assert!(raster.pixels.chunks_exact(4).any(|pixel| pixel[3] == 255));
    assert!(raster.pixels.chunks_exact(4).any(|pixel| pixel[3] == 0));
  }

  #[test]
  fn legacy_modifier_masks_expand_into_separate_caps() {
    let mut overlay = overlay_with(&[35]);
    overlay.keys[0].modifier_mask = 0b0000_0011;

    let prepared = prepared_shortcut(&overlay);

    assert_eq!(
      prepared.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
      vec![55, 59, 35]
    );
  }

  #[test]
  fn backing_scale_covers_the_animated_excursion_within_its_clamp() {
    let overlay = overlay_with(&[35]);

    assert_eq!(keyboard_backing_scale(1080, &overlay), 12.0);
    assert_eq!(keyboard_backing_scale(8640, &overlay), 26.0);
    assert_eq!(keyboard_backing_scale(u32::MAX, &overlay), 64.0);
  }
}
