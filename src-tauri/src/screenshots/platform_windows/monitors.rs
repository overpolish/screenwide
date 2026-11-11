// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every monitor frozen at once, for the overlays that draw over a still of
//! the desktop. Each Windows Graphics Capture session takes a while to hand
//! over its first frame, most on a slow graphics chip, and `xcap` captures one
//! monitor at a time behind a lock of its own. Here every monitor's session
//! starts together, so the wait is the slowest monitor's rather than all of
//! them in turn. Each session ends as soon as its frame has arrived.

use std::ffi::c_void;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

use windows::core::{factory, IInspectable, Interface};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
  Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
  D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
  D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
  D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::System::WinRT::Direct3D11::{
  CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;

use crate::screenshots::CapturedImage;

/// How long every monitor together may take to hand over a frame.
const FIRST_FRAMES: Duration = Duration::from_secs(3);

fn text(error: windows::core::Error) -> String {
  error.to_string()
}

/// One monitor's capture, stopped when dropped.
struct Capture {
  pool: Direct3D11CaptureFramePool,
  session: GraphicsCaptureSession,
}

impl Drop for Capture {
  fn drop(&mut self) {
    let _ = self.session.Close();
    let _ = self.pool.Close();
  }
}

struct Device {
  d3d: ID3D11Device,
  context: ID3D11DeviceContext,
  winrt: IDirect3DDevice,
}

/// A still of each monitor in `monitor_ids`, in the same order. The monitors'
/// ids are their `HMONITOR`s, as `xcap` numbers them.
pub fn capture(monitor_ids: &[u32]) -> Vec<Result<CapturedImage, String>> {
  super::menu_fade::wait();
  let device = match device() {
    Ok(device) => device,
    Err(error) => return monitor_ids.iter().map(|_| Err(error.clone())).collect(),
  };
  let (arrived, frames) = channel();
  let mut captures = monitor_ids
    .iter()
    .enumerate()
    .map(|(index, &id)| {
      let arrived = arrived.clone();
      start(&device, id, move || {
        let _ = arrived.send(index);
      })
    })
    .collect::<Vec<_>>();
  drop(arrived);
  let mut stills = captures
    .iter()
    .map(|capture| capture.as_ref().err().map(|error| Err(error.clone())))
    .collect::<Vec<Option<Result<CapturedImage, String>>>>();
  collect(&device, &mut captures, &mut stills, &frames);
  stills
    .into_iter()
    .map(|still| {
      still.unwrap_or_else(|| Err("The monitor did not hand over a picture in time".to_owned()))
    })
    .collect()
}

/// Reads each monitor's first frame as it arrives, until every capture has
/// one or [`FIRST_FRAMES`] has passed. Reading happens here, on one thread,
/// as the device's context is not to be used from several at once.
fn collect(
  device: &Device,
  captures: &mut [Result<Capture, String>],
  stills: &mut [Option<Result<CapturedImage, String>>],
  frames: &Receiver<usize>,
) {
  let deadline = Instant::now() + FIRST_FRAMES;
  while stills.iter().any(Option::is_none) {
    let Some(left) = deadline.checked_duration_since(Instant::now()) else {
      return;
    };
    let Ok(index) = frames.recv_timeout(left) else {
      return;
    };
    if stills[index].is_some() {
      continue;
    }
    if let Ok(capture) = &captures[index] {
      stills[index] = Some(read(device, &capture.pool));
    }
    // Ends the session at once, and its border with it.
    captures[index] = Err(String::new());
  }
}

/// A plain Direct3D 11 device on the default adapter, which Windows
/// Graphics Capture hands its frames to.
fn device() -> Result<Device, String> {
  let mut d3d = None;
  let mut context = None;
  unsafe {
    D3D11CreateDevice(
      None,
      D3D_DRIVER_TYPE_HARDWARE,
      HMODULE::default(),
      D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      None,
      D3D11_SDK_VERSION,
      Some(&mut d3d),
      None,
      Some(&mut context),
    )
  }
  .map_err(|error| format!("The capture device could not be opened: {error}"))?;
  let d3d: ID3D11Device = d3d.ok_or("Direct3D 11 returned no capture device")?;
  let context = context.ok_or("Direct3D 11 returned no capture device context")?;
  let dxgi = d3d.cast::<IDXGIDevice>().map_err(text)?;
  let winrt = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
    .map_err(text)?
    .cast::<IDirect3DDevice>()
    .map_err(text)?;
  Ok(Device {
    d3d,
    context,
    winrt,
  })
}

/// Starts capturing the monitor `id`, calling `arrived` as each frame comes.
fn start(
  device: &Device,
  id: u32,
  arrived: impl Fn() + Send + Sync + 'static,
) -> Result<Capture, String> {
  let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>().map_err(text)?;
  let item: GraphicsCaptureItem =
    unsafe { interop.CreateForMonitor(HMONITOR(id as usize as *mut c_void)) }
      .map_err(|_| "The selected monitor is no longer available".to_owned())?;
  let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
    &device.winrt,
    DirectXPixelFormat::B8G8R8A8UIntNormalized,
    1,
    item.Size().map_err(text)?,
  )
  .map_err(text)?;
  pool
    .FrameArrived(
      &TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(move |_, _| {
        arrived();
        Ok(())
      }),
    )
    .map_err(text)?;
  let session = pool.CreateCaptureSession(&item).map_err(text)?;
  let capture = Capture { pool, session };
  capture
    .session
    .SetIsCursorCaptureEnabled(false)
    .map_err(text)?;
  // Ignored where Windows has no borderless capture, or the person declined
  // it: the border shows, and the capture goes on.
  let _ = capture.session.SetIsBorderRequired(false);
  capture.session.StartCapture().map_err(text)?;
  Ok(capture)
}

/// The frame waiting in `pool`, copied out of the GPU as red, green, blue and
/// alpha bytes.
fn read(device: &Device, pool: &Direct3D11CaptureFramePool) -> Result<CapturedImage, String> {
  let frame = pool.TryGetNextFrame().map_err(text)?;
  let texture = unsafe {
    frame
      .Surface()
      .map_err(text)?
      .cast::<IDirect3DDxgiInterfaceAccess>()
      .map_err(text)?
      .GetInterface::<ID3D11Texture2D>()
  }
  .map_err(text)?;
  let mut description = D3D11_TEXTURE2D_DESC::default();
  unsafe { texture.GetDesc(&mut description) };
  let staging = {
    let staging_description = D3D11_TEXTURE2D_DESC {
      BindFlags: 0,
      MiscFlags: 0,
      Usage: D3D11_USAGE_STAGING,
      CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
      ..description
    };
    let mut staging = None;
    unsafe {
      device
        .d3d
        .CreateTexture2D(&staging_description, None, Some(&mut staging))
    }
    .map_err(text)?;
    let staging: ID3D11Texture2D = staging.ok_or("Direct3D 11 made no copy of the frame")?;
    staging.cast::<ID3D11Resource>().map_err(text)?
  };
  let source = texture.cast::<ID3D11Resource>().map_err(text)?;
  unsafe { device.context.CopyResource(&staging, &source) };
  let _ = frame.Close();
  let (width, height) = (description.Width, description.Height);
  let row = width as usize * 4;
  let mut rgba = vec![0_u8; row * height as usize];
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  unsafe {
    device
      .context
      .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
  }
  .map_err(text)?;
  for (y, target) in rgba.chunks_exact_mut(row).enumerate() {
    let source = unsafe {
      std::slice::from_raw_parts(
        (mapped.pData as *const u8).add(y * mapped.RowPitch as usize),
        row,
      )
    };
    for (to, from) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
      to.copy_from_slice(&[from[2], from[1], from[0], from[3]]);
    }
  }
  unsafe { device.context.Unmap(&staging, 0) };
  Ok(CapturedImage {
    rgba,
    width,
    height,
  })
}
