// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Device {
  /// The BGRA-capable D3D11 device, the DXGI factory reached through its
  /// adapter, and the DirectComposition device that shares it. Multithread
  /// protection is on because overlay work is dispatched between the thread
  /// that owns a window and the one that drives a session.
  pub(crate) fn new() -> Result<Arc<Self>, String> {
    let mut device = None;
    let mut context = None;
    unsafe {
      D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        Some(&mut context),
      )
    }
    .map_err(|error| format!("The Windows overlay GPU could not be opened: {error}"))?;
    let device = device.ok_or_else(|| "D3D11 returned no overlay device".to_owned())?;
    let context = context.ok_or_else(|| "D3D11 returned no overlay context".to_owned())?;
    let multithread: ID3D10Multithread = device.cast().map_err(|error| error.to_string())?;
    let _ = unsafe { multithread.SetMultithreadProtected(true) };
    let dxgi: IDXGIDevice = device.cast().map_err(|error| error.to_string())?;
    let adapter: IDXGIAdapter = unsafe { dxgi.GetAdapter() }.map_err(|error| error.to_string())?;
    let factory: IDXGIFactory2 =
      unsafe { adapter.GetParent() }.map_err(|error| error.to_string())?;
    let composition: IDCompositionDevice = unsafe { DCompositionCreateDevice(&dxgi) }
      .map_err(|error| format!("DirectComposition could not use the overlay GPU: {error}"))?;
    Ok(Arc::new(Self {
      device,
      context,
      factory,
      composition,
    }))
  }

  pub(crate) fn device(&self) -> &ID3D11Device {
    &self.device
  }

  pub(crate) fn context(&self) -> &ID3D11DeviceContext {
    &self.context
  }

  /// Publishes changes to the composition tree. Presenting a swap chain is
  /// enough once its visual is committed; a tree that has just changed is not
  /// on screen until this returns.
  pub(crate) fn commit(&self) -> Result<(), String> {
    unsafe { self.composition.Commit() }.map_err(|error| error.to_string())
  }
}
