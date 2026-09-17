// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows plumbing every DirectComposition overlay is built from.
//!
//! A transparent always-on-top overlay is the same three pieces whatever it
//! draws: one D3D11 device the feature's surfaces share, one
//! `WS_EX_NOREDIRECTIONBITMAP` child window per host whose pixels
//! DirectComposition owns, and one composition swap chain per child. The
//! region OSC and live annotation both stand on these; the pipeline and the
//! scene drawn through them belong to the feature.

mod device;
mod swap_chain;
mod window;

pub(crate) use window::{
  create_child, create_on_owning_thread, disable_transitions, register_class, set_capture_affinity,
};

use std::sync::Arc;

use windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HMODULE, HWND},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1},
      Direct3D10::ID3D10Multithread,
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
        ID3D11Resource, ID3D11Texture2D, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED},
      Dxgi::{
        Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISwapChain3, DXGI_PRESENT,
        DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
        DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
      },
    },
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::{
      CreateWindowExW, GetWindowThreadProcessId, LoadCursorW, RegisterClassW,
      SetWindowDisplayAffinity, HMENU, IDC_ARROW, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WNDCLASSW,
      WNDCLASS_STYLES, WNDPROC, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP,
    },
  },
};

/// The device a feature's overlay surfaces share, so a second display costs
/// one window and one swap chain rather than a second device.
pub(crate) struct Device {
  device: ID3D11Device,
  context: ID3D11DeviceContext,
  factory: IDXGIFactory2,
  composition: IDCompositionDevice,
}

/// One child window's composition target: the swap chain its pixels arrive
/// through, and the visual tree that puts them on screen.
pub(crate) struct CompositionSwapChain {
  chain: IDXGISwapChain3,
  size: (u32, u32),
  /// Held only to keep the composition tree alive; nothing is mutated after
  /// construction.
  _target: IDCompositionTarget,
  _root: IDCompositionVisual,
  _visual: IDCompositionVisual,
}

// The COM interfaces and window handles these own are process-wide tokens.
// Each consumer reaches them behind its own registry lock and touches the
// window from the thread that created it, and the device is multithread
// protected, so the handles travel between threads while the calls stay
// serialized.
unsafe impl Send for Device {}
unsafe impl Sync for Device {}
unsafe impl Send for CompositionSwapChain {}
unsafe impl Sync for CompositionSwapChain {}
