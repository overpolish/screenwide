// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Media Foundation supplies an absolute timestamp derived from the sample's
/// native presentation time. Map it back onto Rust's monotonic clock so audio
/// and video describe when capture occurred, not when webcam decoding ended.
/// Implausible/missing device clocks retain the arrival-time fallback.
pub(super) fn camera_frame_clock(
  captured_epoch: Option<Duration>,
  arrived: Instant,
  arrival_epoch: Option<Duration>,
  stream_started: Instant,
) -> (Instant, i64) {
  const MAX_CAPTURE_LATENCY: Duration = Duration::from_secs(2);
  if let (Some(captured_epoch), Some(arrival_epoch)) = (captured_epoch, arrival_epoch) {
    if let Some(age) = arrival_epoch.checked_sub(captured_epoch) {
      if age <= MAX_CAPTURE_LATENCY {
        let captured = arrived.checked_sub(age).unwrap_or(arrived);
        let source_100ns = i64::try_from(captured_epoch.as_nanos() / 100).unwrap_or(i64::MAX);
        return (captured, source_100ns);
      }
    }
  }
  (
    arrived,
    i64::try_from(arrived.saturating_duration_since(stream_started).as_nanos() / 100)
      .unwrap_or(i64::MAX),
  )
}

pub(super) fn report_once(reported: &AtomicBool, report: &FailureReport, message: String) {
  if !reported.swap(true, Ordering::AcqRel) {
    report(format!("Camera recording failed: {message}"));
  }
}

pub(super) fn texture(
  device: &ID3D11Device,
  width: u32,
  height: u32,
  bgra: &[u8],
) -> Result<ID3D11Texture2D, String> {
  let expected = width as usize * height as usize * 4;
  if bgra.len() != expected {
    return Err("The camera produced an incomplete video frame".to_owned());
  }
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    // Matches the screen-capture textures the sink writer already accepts. A
    // shader-resource-only texture is rejected by the Media Foundation video
    // pipeline with E_INVALIDARG when the sample reaches the encoder.
    BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
    ..Default::default()
  };
  let initial = D3D11_SUBRESOURCE_DATA {
    pSysMem: bgra.as_ptr().cast(),
    SysMemPitch: width * 4,
    ..Default::default()
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  texture.ok_or_else(|| "D3D11 created no camera texture".to_owned())
}

pub(super) fn bgra_pixels(rgba: &[u8], width: u32, height: u32, flipped: bool) -> Vec<u8> {
  let stride = width as usize * 4;
  let mut bgra = vec![0_u8; stride * height as usize];
  bgra
    .par_chunks_mut(stride)
    .enumerate()
    .for_each(|(row, output)| {
      let input = &rgba[row * stride..(row + 1) * stride];
      for x in 0..width as usize {
        let source_x = if flipped { width as usize - 1 - x } else { x };
        let source = &input[source_x * 4..source_x * 4 + 4];
        let target = &mut output[x * 4..x * 4 + 4];
        target.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
      }
    });
  bgra
}
