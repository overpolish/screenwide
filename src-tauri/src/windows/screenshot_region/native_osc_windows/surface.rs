// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region OSC surfaces. The anchor display draws into a
//! `WS_EX_NOREDIRECTIONBITMAP` child of the Tauri window; every other display
//! gets a `WS_POPUP` peer, the Win32 twin of the macOS `NSPanel` peers. All of
//! them share one [`Gpu`] — one D3D11 device, one DirectComposition device and
//! one pipeline — and own only their window, target, visual and swap chain.
//!
//! Port of `screenshot_region_osc_macos.m`'s attach and master frame draw
//! (`:39-174`), `+desktop.m` and `+snapshot.m`, minus OCR and ruler.

use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use windows::{
  core::{s, w, Interface, PCWSTR},
  Win32::{
    Foundation::{COLORREF, ERROR_SUCCESS, HINSTANCE, HMODULE, HWND, POINT, RECT},
    Graphics::{
      Direct3D::{
        D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
        D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
      },
      Direct3D10::ID3D10Multithread,
      Direct3D11::{
        D3D11CreateDevice, ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext,
        ID3D11InputLayout, ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView,
        ID3D11Resource, ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D,
        ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE,
        D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE,
        D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL,
        D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CULL_NONE,
        D3D11_FILL_SOLID, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_FILTER_MIN_MAG_MIP_POINT,
        D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAPPED_SUBRESOURCE,
        D3D11_MAP_WRITE_DISCARD, D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC,
        D3D11_SAMPLER_DESC, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
        D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED},
      Dxgi::{
        Common::{
          DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT,
          DXGI_FORMAT_R32_UINT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8_UNORM, DXGI_SAMPLE_DESC,
        },
        IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISwapChain3, DXGI_PRESENT,
        DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
        DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
      },
      Gdi::ScreenToClient,
    },
    System::{
      LibraryLoader::GetModuleHandleW,
      Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
      Threading::GetCurrentThreadId,
    },
    UI::{
      HiDpi::GetDpiForWindow,
      WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetClientRect, GetCursorPos, GetWindowLongPtrW,
        GetWindowRect, GetWindowThreadProcessId, KillTimer, LoadCursorW, RegisterClassW, SetCursor,
        SetLayeredWindowAttributes, SetTimer, SetWindowDisplayAffinity, SetWindowLongPtrW,
        SetWindowPos, ShowWindowAsync, CS_DBLCLKS, GWL_EXSTYLE, HMENU, HWND_TOP, HWND_TOPMOST,
        IDC_ARROW, IDC_CROSS, LWA_ALPHA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
        SW_HIDE, SW_SHOWNOACTIVATE, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WNDCLASSW, WS_CHILD,
        WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
      },
    },
  },
};

use super::input;
use super::ocr::{self, Segment};
use super::renderer::{self, RenderConstants, Vertex, PIXEL_SHADER, VERTEX_SHADER};
use super::ruler;
use crate::osc::geometry::{Point, Rect, Size};

/// Windows reports DPI relative to this baseline.
const BASE_DPI: f64 = 96.0;

static OVERLAY_CLASS: OnceLock<u16> = OnceLock::new();
static PEER_CLASS: OnceLock<u16> = OnceLock::new();

/// The lens anchor and dragged-edge bitmask. The sampling window and box size
/// are fixed, so a frame can rebuild the constants from these two values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MagnifierAnchor {
  pub point: Point,
  pub edges: u32,
}

struct Texture {
  view: ID3D11ShaderResourceView,
  size: (u32, u32),
}

/// Everything shared by the anchor surface and its peers, so a peer costs one
/// window plus one swap chain. macOS shared the `MTLDevice` the same way.
pub(crate) struct Gpu {
  device: ID3D11Device,
  context: ID3D11DeviceContext,
  factory: IDXGIFactory2,
  composition: IDCompositionDevice,
  vertex_shader: ID3D11VertexShader,
  pixel_shader: ID3D11PixelShader,
  layout: ID3D11InputLayout,
  constants: ID3D11Buffer,
  rasterizer: ID3D11RasterizerState,
  blend: ID3D11BlendState,
  /// The frozen-desktop variant: `srcA = ONE`, preserving opaque target alpha
  /// while translucent chrome is drawn above the snapshot.
  opaque_blend: ID3D11BlendState,
  linear_sampler: ID3D11SamplerState,
  point_sampler: ID3D11SamplerState,
  placeholder: ID3D11ShaderResourceView,
  /// The shared control-icon atlas, bound at t2 for every frame.
  icons: ID3D11ShaderResourceView,
}

impl Gpu {
  pub(super) fn device(&self) -> &ID3D11Device {
    &self.device
  }
}

// Surfaces are reached through the context registry from the UI thread; the
// COM interfaces and window handles they own are process-wide tokens guarded
// by the context mutex, and the device is multithread-protected.
unsafe impl Send for Gpu {}
unsafe impl Sync for Gpu {}
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Gpu {
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
    .map_err(|error| format!("The Windows region OSC GPU could not be opened: {error}"))?;
    let device = device.ok_or_else(|| "D3D11 returned no region OSC device".to_owned())?;
    let context = context.ok_or_else(|| "D3D11 returned no region OSC context".to_owned())?;
    let multithread: ID3D10Multithread = device.cast().map_err(|error| error.to_string())?;
    let _ = unsafe { multithread.SetMultithreadProtected(true) };
    let dxgi: IDXGIDevice = device.cast().map_err(|error| error.to_string())?;
    let adapter: IDXGIAdapter = unsafe { dxgi.GetAdapter() }.map_err(|error| error.to_string())?;
    let factory: IDXGIFactory2 =
      unsafe { adapter.GetParent() }.map_err(|error| error.to_string())?;
    let composition: IDCompositionDevice = unsafe { DCompositionCreateDevice(&dxgi) }
      .map_err(|error| format!("DirectComposition could not use the region OSC GPU: {error}"))?;

    let mut vertex_shader = None;
    let mut pixel_shader = None;
    let mut layout = None;
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreateInputLayout(&input_elements(), VERTEX_SHADER, Some(&mut layout))
        .map_err(|error| error.to_string())?;
    }
    let mut constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<RenderConstants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut constants),
      )
    }
    .map_err(|error| error.to_string())?;

    // Metal's normal pipeline: straight source-over on colour and alpha.
    let blend = blend_state(&device, D3D11_BLEND_SRC_ALPHA)?;
    let opaque_blend = blend_state(&device, D3D11_BLEND_ONE)?;
    // Metal's default is no face culling. Several shared OSC primitives are
    // deliberately emitted in either winding (notably line quads), so D3D's
    // default back-face culling silently removed rulers and probes.
    let mut rasterizer = None;
    unsafe {
      device.CreateRasterizerState(
        &D3D11_RASTERIZER_DESC {
          FillMode: D3D11_FILL_SOLID,
          CullMode: D3D11_CULL_NONE,
          DepthClipEnable: true.into(),
          ..Default::default()
        },
        Some(&mut rasterizer),
      )
    }
    .map_err(|error| error.to_string())?;
    let linear_sampler = sampler(&device, D3D11_FILTER_MIN_MAG_MIP_LINEAR)?;
    let point_sampler = sampler(&device, D3D11_FILTER_MIN_MAG_MIP_POINT)?;
    // No SRV slot may ever be null; t0-t4 fall back to one transparent texel.
    let placeholder = upload_rgba(&device, &[0_u8; 4], 1, 1)?;
    let icons = upload_icons(&device).unwrap_or_else(|error| {
      eprintln!("The Windows region OSC could not upload the icon atlas: {error}");
      placeholder.clone()
    });

    Ok(Arc::new(Self {
      device,
      context,
      factory,
      composition,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no region OSC vertex shader".to_owned())?,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no region OSC pixel shader".to_owned())?,
      layout: layout.ok_or_else(|| "D3D11 created no region OSC input layout".to_owned())?,
      constants: constants.ok_or_else(|| "D3D11 created no region OSC constants".to_owned())?,
      rasterizer: rasterizer.ok_or_else(|| "D3D11 created no region OSC rasterizer".to_owned())?,
      blend,
      opaque_blend,
      linear_sampler,
      point_sampler,
      placeholder,
      icons,
    }))
  }
}

/// Where a surface takes its geometry and DPI from.
enum Kind {
  /// The anchor display: a child of the Tauri window, sized to its client area.
  Root { host: HWND },
  /// One non-anchor display: a top-level popup covering its monitor. Its scale
  /// comes from the binding because a peer may sit on a different-DPI monitor.
  Peer { bounds: Rect, scale: f64 },
}

pub(crate) struct Surface {
  gpu: Arc<Gpu>,
  kind: Kind,
  hwnd: HWND,
  swap_chain: IDXGISwapChain3,
  /// Held only to keep the composition tree alive; nothing is mutated after
  /// construction.
  _target: IDCompositionTarget,
  _root: IDCompositionVisual,
  _visual: IDCompositionVisual,
  vertex_buffer: Option<ID3D11Buffer>,
  vertex_capacity: usize,
  vertices: Vec<Vertex>,
  buffer_size: (u32, u32),
  magnifier_source: Option<Texture>,
  snapshot: Option<Texture>,

  // Presentation mirror — the fields `+state.m` kept on the ObjC object.
  pub(crate) display_id: u32,
  pub(crate) region: Rect,
  pub(crate) visible: bool,
  pub(crate) show_frame: bool,
  pub(crate) show_handles: bool,
  pub(crate) input_enabled: bool,
  pub(crate) exclusion_rect: Rect,
  pub(crate) magnifier: Option<MagnifierAnchor>,
  pub(crate) snapshot_presented: bool,
  pub(crate) snapshot_composited: bool,
  /// Peers are only ordered on screen while the desktop is presented; the root
  /// ignores this, its window is the webview's own.
  pub(crate) desktop_presented: bool,
  /// This surface's origin in the desktop plane. `region` is desktop-global, so
  /// drawing subtracts the offset and input adds it — the macOS
  /// `desktopOffset`.
  desktop_offset: Point,
  pub(crate) gesture_active: bool,
  pub(crate) cursor: input::CursorShape,
  /// The OCR overlay this surface hosts: highlights plus the chrome that used
  /// to live in its own material surfaces.
  pub(crate) ocr: ocr::Chrome,
  /// The Ruler overlay this surface hosts: world artifacts, the pooled labels
  /// and the cursor readout.
  pub(crate) ruler: ruler::Ruler,
  animating: bool,
  shown: bool,
  window_size: (u32, u32),
  drawing: bool,
  pending: bool,
}

impl Surface {
  pub(crate) fn root(gpu: Arc<Gpu>, host: HWND, overlay: HWND) -> Result<Self, String> {
    // The overlay child is `WS_EX_NOREDIRECTIONBITMAP`, so DirectComposition
    // owns all of its content and the target is created topmost — that is what
    // keeps the region frame above the WebView2 sibling.
    Self::new(gpu, Kind::Root { host }, overlay, 0)
  }

  pub(crate) fn peer(
    gpu: Arc<Gpu>,
    hwnd: HWND,
    display_id: u32,
    bounds: Rect,
    scale: f64,
  ) -> Result<Self, String> {
    Self::new(gpu, Kind::Peer { bounds, scale }, hwnd, display_id)
  }

  fn new(gpu: Arc<Gpu>, kind: Kind, hwnd: HWND, display_id: u32) -> Result<Self, String> {
    let target = unsafe { gpu.composition.CreateTargetForHwnd(hwnd, true) }
      .map_err(|error| format!("The Windows region OSC could not attach: {error}"))?;
    let root = unsafe { gpu.composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe { target.SetRoot(&root) }.map_err(|error| error.to_string())?;
    let description = DXGI_SWAP_CHAIN_DESC1 {
      Width: 2,
      Height: 2,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
      AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
      ..Default::default()
    };
    let swap_chain = unsafe {
      gpu
        .factory
        .CreateSwapChainForComposition(&gpu.device, &description, None)
    }
    .and_then(|chain| chain.cast::<IDXGISwapChain3>())
    .map_err(|error| format!("The Windows region OSC swap chain failed: {error}"))?;
    let visual = unsafe { gpu.composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe {
      visual
        .SetContent(&swap_chain)
        .map_err(|error| error.to_string())?;
      // DirectComposition defaults to nearest-neighbour bitmap sampling.
      visual
        .SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)
        .map_err(|error| error.to_string())?;
      root
        .AddVisual(&visual, true, None::<&IDCompositionVisual>)
        .map_err(|error| error.to_string())?;
      gpu
        .composition
        .Commit()
        .map_err(|error| error.to_string())?;
    }
    Ok(Self {
      gpu,
      kind,
      hwnd,
      swap_chain,
      _target: target,
      _root: root,
      _visual: visual,
      vertex_buffer: None,
      vertex_capacity: 0,
      vertices: Vec::new(),
      buffer_size: (2, 2),
      magnifier_source: None,
      snapshot: None,
      display_id,
      region: Rect::default(),
      visible: false,
      show_frame: true,
      show_handles: true,
      input_enabled: false,
      exclusion_rect: Rect::default(),
      magnifier: None,
      snapshot_presented: false,
      snapshot_composited: false,
      desktop_presented: false,
      desktop_offset: Point::default(),
      gesture_active: false,
      cursor: input::CursorShape::None,
      ocr: ocr::Chrome::default(),
      ruler: ruler::Ruler::default(),
      animating: false,
      shown: false,
      window_size: (0, 0),
      drawing: false,
      pending: false,
    })
  }

  pub(crate) fn hwnd(&self) -> HWND {
    self.hwnd
  }

  pub(crate) fn is_root(&self) -> bool {
    matches!(self.kind, Kind::Root { .. })
  }

  /// The peer's monitor rectangle in physical pixels, for diffing a rebuild.
  pub(crate) fn peer_geometry(&self) -> Option<(Rect, f64)> {
    match self.kind {
      Kind::Root { .. } => None,
      Kind::Peer { bounds, scale } => Some((bounds, scale)),
    }
  }

  /// Physical pixels per logical point.
  fn scale(&self) -> f64 {
    match self.kind {
      Kind::Root { host } => {
        let dpi = unsafe { GetDpiForWindow(host) };
        if dpi == 0 {
          1.0
        } else {
          f64::from(dpi) / BASE_DPI
        }
      }
      Kind::Peer { scale, .. } => scale,
    }
  }

  /// Client size in physical pixels.
  fn client_size(&self) -> (u32, u32) {
    let window = match self.kind {
      Kind::Root { host } => host,
      Kind::Peer { .. } => self.hwnd,
    };
    let mut rect = RECT::default();
    if unsafe { GetClientRect(window, &mut rect) }.is_err() {
      return (0, 0);
    }
    (
      (rect.right - rect.left).max(0) as u32,
      (rect.bottom - rect.top).max(0) as u32,
    )
  }

  /// Converts a client-space physical point to surface-local logical points.
  pub(crate) fn logical_point(&self, x: f64, y: f64) -> Point {
    let scale = self.scale().max(0.1);
    Point {
      x: x / scale,
      y: y / scale,
    }
  }

  /// Surface-local logical points lifted into the desktop plane, which is what
  /// the controller and the semantic events are configured in.
  pub(crate) fn desktop_point(&self, point: Point) -> Point {
    Point {
      x: point.x + self.desktop_offset.x,
      y: point.y + self.desktop_offset.y,
    }
  }

  /// The reverse: a desktop-global rect in this surface's own coordinates.
  pub(crate) fn local_rect(&self, rect: Rect) -> Rect {
    Rect {
      origin: Point {
        x: rect.origin.x - self.desktop_offset.x,
        y: rect.origin.y - self.desktop_offset.y,
      },
      size: rect.size,
    }
  }

  pub(crate) fn set_desktop_offset(&mut self, origin: Point) {
    if self.desktop_offset != origin {
      self.desktop_offset = origin;
      self.draw();
    }
  }

  /// True when the pointer is over this surface's window.
  pub(crate) fn contains_screen_point(&self, point: POINT) -> bool {
    let window = match self.kind {
      Kind::Root { host } => host,
      Kind::Peer { .. } => self.hwnd,
    };
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(window, &mut rect) }.is_err() {
      return false;
    }
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
  }

  pub(crate) fn screen_to_client(&self, x: i32, y: i32) -> (f64, f64) {
    let mut point = POINT { x, y };
    let _ = unsafe { ScreenToClient(self.hwnd, &mut point) };
    (f64::from(point.x), f64::from(point.y))
  }

  pub(crate) fn set_region(&mut self, region: Rect, visible: bool) {
    self.region = region;
    self.visible = visible;
    if !visible {
      self.magnifier = None;
    }
    self.draw();
  }

  pub(crate) fn set_magnifier_source(&mut self, rgba: &[u8], width: u32, height: u32) -> bool {
    match self.upload(rgba, width, height) {
      Some(texture) => {
        self.magnifier_source = Some(texture);
        self.draw();
        true
      }
      None => false,
    }
  }

  pub(crate) fn has_magnifier_source(&self) -> bool {
    self.magnifier_source.is_some()
  }

  /// Port of `+snapshot.m:13-20`: the frozen desktop is per display, so only
  /// the surface owning `display_id` keeps the pixels.
  pub(crate) fn set_snapshot(&mut self, rgba: &[u8], width: u32, height: u32) -> bool {
    match self.upload(rgba, width, height) {
      Some(texture) => {
        self.snapshot = Some(texture);
        self.draw();
        true
      }
      None => false,
    }
  }

  fn upload(&self, rgba: &[u8], width: u32, height: u32) -> Option<Texture> {
    let expected = width as usize * height as usize * 4;
    if width == 0 || height == 0 || rgba.len() != expected {
      return None;
    }
    // The caller only lends the buffer for this call, so it is copied into a
    // texture before returning.
    match upload_rgba(&self.gpu.device, rgba, width, height) {
      Ok(view) => Some(Texture {
        view,
        size: (width, height),
      }),
      Err(error) => {
        eprintln!("The Windows region OSC could not upload a texture: {error}");
        None
      }
    }
  }

  /// Raises this surface and takes cursor ownership for the pointer.
  pub(crate) fn claim_pointer(&mut self) {
    self.raise();
    self.cursor = input::CursorShape::Crosshair;
    if let Ok(cursor) = unsafe { LoadCursorW(None, IDC_CROSS) } {
      unsafe { SetCursor(Some(cursor)) };
    }
  }

  pub(crate) fn release_pointer(&mut self) {
    self.cursor = input::CursorShape::None;
  }

  /// Keeps the root overlay sized to the host client area and above its
  /// WebView2 sibling, and each peer covering its own monitor above everything
  /// else — the Win32 form of `orderFrontRegardless` with the parent's level.
  fn raise(&self) {
    match self.kind {
      Kind::Root { .. } => {
        let (width, height) = self.client_size();
        let _ = unsafe {
          SetWindowPos(
            self.hwnd,
            Some(HWND_TOP),
            0,
            0,
            width.max(1) as i32,
            height.max(1) as i32,
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
          )
        };
      }
      Kind::Peer { bounds, .. } => {
        let _ = unsafe {
          SetWindowPos(
            self.hwnd,
            Some(HWND_TOPMOST),
            bounds.origin.x as i32,
            bounds.origin.y as i32,
            (bounds.size.width as i32).max(1),
            (bounds.size.height as i32).max(1),
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
          )
        };
      }
    }
  }

  pub(crate) fn draw(&mut self) {
    if self.drawing {
      self.pending = true;
      return;
    }
    self.drawing = true;
    loop {
      self.pending = false;
      if let Err(error) = self.render() {
        eprintln!("The Windows region OSC frame was dropped: {error}");
      }
      if !self.pending {
        break;
      }
    }
    self.drawing = false;
  }

  fn render(&mut self) -> Result<(), String> {
    let (width, height) = self.client_size();
    if !should_show(self.is_root(), self.visible, self.desktop_presented)
      || width == 0
      || height == 0
    {
      if self.shown {
        self.shown = false;
        let _ = unsafe { ShowWindowAsync(self.hwnd, SW_HIDE) };
      }
      return Ok(());
    }
    // Layout is only re-asserted when it actually changes: a pointer move
    // must not post a `SetWindowPos` per frame.
    if !self.shown || self.window_size != (width, height) {
      self.shown = true;
      self.window_size = (width, height);
      let _ = unsafe { ShowWindowAsync(self.hwnd, SW_SHOWNOACTIVATE) };
      self.raise();
    }

    let scale = self.scale().max(0.1);
    // Geometry is built in logical points, exactly as the macOS master frame
    // passed `host.bounds.size`; only the magnifier quad works in the physical
    // pixels its constants and `SV_Position` use.
    let view = Size {
      width: f64::from(width) / scale,
      height: f64::from(height) / scale,
    };
    let pixels = Size {
      width: f64::from(width),
      height: f64::from(height),
    };
    let canvas = Rect::from_xywh(0.0, 0.0, view.width, view.height);

    let light = light_mode();
    let now = std::time::Instant::now();
    let mut constants = RenderConstants::new(light);
    // The ruler animation row is frame-global: the world halos read its hover
    // alpha and width, and the loupe reads the copied and tolerance progress.
    constants.ruler_sample = [
      f32::from((self.ruler.color >> 24) as u8) / 255.0,
      f32::from((self.ruler.color >> 16) as u8) / 255.0,
      f32::from((self.ruler.color >> 8) as u8) / 255.0,
      f32::from(self.ruler.color as u8) / 255.0,
    ];
    constants.ruler_animation = [
      self.ruler.copied_amount(now) as f32,
      self.ruler.hover_alpha() as f32,
      (self.ruler.hover_width(now) * scale) as f32,
      self.ruler.tolerance_amount(now) as f32,
    ];
    // The lens belongs to the surface the pointer is on; peers never carry an
    // anchor (`updateMagnifier` required `s == root`).
    let magnifier_active = self.show_frame
      && self.input_enabled
      && self.magnifier.is_some()
      && self.magnifier_source.is_some();
    if magnifier_active {
      if let (Some(anchor), Some(source)) = (self.magnifier, self.magnifier_source.as_ref()) {
        constants.set_magnifier(
          anchor.point,
          scale,
          anchor.edges,
          source.size,
          (
            (anchor.point.x / view.width.max(1.0)) as f32,
            (anchor.point.y / view.height.max(1.0)) as f32,
          ),
          (0.0, 0.0),
          (1.0, 1.0),
        );
      }
    } else {
      constants.clear_magnifier();
    }

    let mut vertices = std::mem::take(&mut self.vertices);
    vertices.clear();
    let mut segments: Vec<Segment> = Vec::new();
    // macOS drew the non-composited snapshot through a `CALayer` under the
    // Metal layer. One full-viewport quad in the same pass is visually
    // equivalent and saves a second DirectComposition visual; only the blend
    // state still distinguishes the composited path.
    let snapshot_drawn = self.snapshot_presented && self.snapshot.is_some();
    let snapshot_source = if snapshot_drawn && self.snapshot_composited {
      self.ruler.snapshot_uv(view)
    } else {
      Rect::from_xywh(0.0, 0.0, 1.0, 1.0)
    };
    if snapshot_drawn {
      let (source_width, source_height) =
        self.snapshot.as_ref().map_or((1, 1), |source| source.size);
      constants.chrome_backdrop = [
        pixels.width as f32,
        pixels.height as f32,
        1.0 / source_width.max(1) as f32,
        1.0 / source_height.max(1) as f32,
      ];
      constants.chrome_source = [
        snapshot_source.origin.x as f32,
        snapshot_source.origin.y as f32,
        snapshot_source.size.width as f32,
        snapshot_source.size.height as f32,
      ];
    }
    if snapshot_drawn {
      // In the composited (Ruler) mode the uv window is the display's zoomed
      // viewport, so panning and zooming move the frozen desktop itself.
      renderer::add_texture_quad(&mut vertices, view, canvas, snapshot_source, 33);
    }
    // `region` is desktop-global; every builder works in surface-local points.
    let region = self.local_rect(self.region);
    if is_empty(region) {
      renderer::add_quad(&mut vertices, view, canvas, 6);
    } else {
      renderer::add_crop_with_handles(
        &mut vertices,
        view,
        region,
        canvas,
        scale,
        self.show_frame,
        self.show_handles,
      );
    }
    // Highlights sit in world space with the region, so they join the base
    // run rather than a chrome draw call of their own.
    self
      .ocr
      .add_world_vertices(&mut vertices, view, region, scale);
    self.ruler.add_world_vertices(
      &mut vertices,
      view,
      scale,
      self.display_id,
      self.desktop_offset,
      now,
    );
    let chrome_start = vertices.len();
    // macOS drew every control into its own material surface; folding them in
    // here costs one draw call per control, because each carries its own fill
    // and foreground in the constant buffer.
    self.ocr.add_chrome_vertices(
      self.gpu.device(),
      &mut vertices,
      &mut segments,
      view,
      region,
      scale,
      light,
    );
    // Ruler labels and the loupe fold in the same way, one segment each.
    self.ruler.add_chrome_vertices(
      self.gpu.device(),
      &mut vertices,
      &mut segments,
      view,
      self.display_id,
      self.desktop_offset,
      scale,
      light,
      now,
    );
    if let Some(base) = Segment::base(0, chrome_start) {
      segments.insert(0, base);
    }
    // The lens is drawn last: nothing may overdraw it, which is why the Metal
    // cutout disappeared with the compute pass.
    let lens_start = vertices.len();
    renderer::add_magnifier(&mut vertices, pixels, &constants);
    segments.extend(Segment::base(lens_start, vertices.len()));
    let result = self.submit(&vertices, &segments, &constants, (width, height));
    self.vertices = vertices;
    // Control transitions, the confirm crossfade and every ruler transition
    // are time based, so a frame is retimed while one is running
    // (`+ocr_cancel.m:92-102`, `+ruler.m:577-587`).
    self.set_animating(self.ocr.is_animating() || self.ruler.is_animating(now));
    result
  }

  /// Drives the 16ms animation frames macOS scheduled with `dispatch_after`.
  fn set_animating(&mut self, animating: bool) {
    if self.animating == animating {
      return;
    }
    self.animating = animating;
    if animating {
      let _ = unsafe { SetTimer(Some(self.hwnd), input::ANIMATION_TIMER, 16, None) };
    } else {
      let _ = unsafe { KillTimer(Some(self.hwnd), input::ANIMATION_TIMER) };
    }
  }

  /// Surface-local logical size, the space every chrome layout works in.
  pub(crate) fn logical_size(&self) -> Size {
    let (width, height) = self.client_size();
    let scale = self.scale().max(0.1);
    Size {
      width: f64::from(width) / scale,
      height: f64::from(height) / scale,
    }
  }

  pub(crate) fn desktop_offset(&self) -> Point {
    self.desktop_offset
  }

  fn submit(
    &mut self,
    vertices: &[Vertex],
    segments: &[Segment],
    constants: &RenderConstants,
    size: (u32, u32),
  ) -> Result<(), String> {
    if size != self.buffer_size {
      unsafe {
        self.swap_chain.ResizeBuffers(
          2,
          size.0,
          size.1,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_SWAP_CHAIN_FLAG(0),
        )
      }
      .map_err(|error| format!("The Windows region OSC could not resize: {error}"))?;
      self.buffer_size = size;
    }
    self.write_vertices(vertices)?;
    let gpu = Arc::clone(&self.gpu);
    let constants_resource: ID3D11Resource = gpu.constants.cast().map_err(|e| e.to_string())?;
    let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() };
    let texture = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(index) }
      .map_err(|error| error.to_string())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut target: Option<ID3D11RenderTargetView> = None;
    unsafe {
      gpu
        .device
        .CreateRenderTargetView(&resource, None, Some(&mut target))
    }
    .map_err(|error| error.to_string())?;
    let target = target.ok_or_else(|| "D3D11 created no region OSC target".to_owned())?;
    let magnifier = self
      .magnifier_source
      .as_ref()
      .map_or_else(|| gpu.placeholder.clone(), |source| source.view.clone());
    let snapshot = self
      .snapshot
      .as_ref()
      .map_or_else(|| gpu.placeholder.clone(), |source| source.view.clone());
    // macOS puts a non-composited OCR snapshot in an opaque CALayer beneath
    // its transparent Metal layer. Windows folds both into this target, so
    // every presented snapshot—not only Ruler's composited one—must retain
    // opaque destination alpha as translucent shading is drawn over it.
    let blend = if opaque_snapshot_target(self.snapshot_presented, self.snapshot.is_some()) {
      &gpu.opaque_blend
    } else {
      &gpu.blend
    };
    unsafe {
      // Flip-discard back buffers are undefined after a present.
      gpu.context.ClearRenderTargetView(&target, &[0.0; 4]);
      gpu.context.OMSetRenderTargets(Some(&[Some(target)]), None);
      gpu
        .context
        .OMSetBlendState(blend, Some(&[0.0; 4]), 0xffff_ffff);
      gpu.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      gpu.context.RSSetState(&gpu.rasterizer);
      gpu.context.IASetInputLayout(&gpu.layout);
      gpu
        .context
        .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      let stride = size_of::<Vertex>() as u32;
      let offset = 0_u32;
      gpu.context.IASetVertexBuffers(
        0,
        1,
        Some(&self.vertex_buffer.clone()),
        Some(&stride),
        Some(&offset),
      );
      gpu.context.VSSetShader(&gpu.vertex_shader, None);
      gpu.context.PSSetShader(&gpu.pixel_shader, None);
      gpu
        .context
        .PSSetConstantBuffers(0, Some(&[Some(gpu.constants.clone())]));
      gpu.context.PSSetSamplers(
        0,
        Some(&[
          Some(gpu.linear_sampler.clone()),
          Some(gpu.point_sampler.clone()),
        ]),
      );
      // One draw call per constant-buffer state. The base scene is a single
      // segment; each folded-in control adds one because its fill, foreground
      // and label texture are its own.
      for segment in segments {
        if segment.count == 0 {
          continue;
        }
        let mut frame = *constants;
        frame.action_fills = segment.action_fills;
        frame.chrome = segment.chrome;
        frame.chrome_outline = segment.chrome_outline;
        gpu.context.UpdateSubresource(
          &constants_resource,
          0,
          None,
          (&raw const frame).cast::<c_void>(),
          0,
          0,
        );
        let label = segment
          .label
          .clone()
          .unwrap_or_else(|| gpu.placeholder.clone());
        let secondary = segment
          .secondary
          .clone()
          .unwrap_or_else(|| gpu.placeholder.clone());
        gpu.context.PSSetShaderResources(
          0,
          Some(&[
            Some(label),
            Some(secondary),
            Some(gpu.icons.clone()),
            Some(snapshot.clone()),
            Some(magnifier.clone()),
          ]),
        );
        gpu.context.Draw(segment.count, segment.start);
      }
      gpu
        .context
        .PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      gpu.context.OMSetRenderTargets(None, None);
      // Never block the pointer thread on the compositor.
      self
        .swap_chain
        .Present(0, DXGI_PRESENT(0))
        .ok()
        .map_err(|error| error.to_string())?;
    }
    Ok(())
  }

  fn write_vertices(&mut self, vertices: &[Vertex]) -> Result<(), String> {
    if vertices.is_empty() {
      return Ok(());
    }
    if self.vertex_buffer.is_none() || self.vertex_capacity < vertices.len() {
      let capacity = vertices.len().next_power_of_two().max(512);
      let mut buffer = None;
      unsafe {
        self.gpu.device.CreateBuffer(
          &D3D11_BUFFER_DESC {
            ByteWidth: (capacity * size_of::<Vertex>()) as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
            ..Default::default()
          },
          None,
          Some(&mut buffer),
        )
      }
      .map_err(|error| format!("The Windows region OSC vertex buffer failed: {error}"))?;
      self.vertex_buffer = buffer;
      self.vertex_capacity = capacity;
    }
    let buffer = self
      .vertex_buffer
      .clone()
      .ok_or_else(|| "D3D11 created no region OSC vertex buffer".to_owned())?;
    let resource: ID3D11Resource = buffer.cast().map_err(|error| error.to_string())?;
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
      self
        .gpu
        .context
        .Map(&resource, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped))
    }
    .map_err(|error| error.to_string())?;
    unsafe {
      std::ptr::copy_nonoverlapping(
        vertices.as_ptr(),
        mapped.pData.cast::<Vertex>(),
        vertices.len(),
      );
      self.gpu.context.Unmap(&resource, 0);
    }
    Ok(())
  }
}

impl Drop for Surface {
  fn drop(&mut self) {
    if self.animating {
      let _ = unsafe { KillTimer(Some(self.hwnd), input::ANIMATION_TIMER) };
    }
    let _ = unsafe { KillTimer(Some(self.hwnd), input::CONFIRM_TIMER) };
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

/// The root's window belongs to the webview and follows the scene alone; a
/// peer is only ordered on screen while the desktop is presented.
pub(crate) fn should_show(is_root: bool, visible: bool, desktop_presented: bool) -> bool {
  visible && (is_root || desktop_presented)
}

fn opaque_snapshot_target(snapshot_presented: bool, has_snapshot: bool) -> bool {
  snapshot_presented && has_snapshot
}

pub(crate) fn cursor_position() -> Option<POINT> {
  let mut point = POINT::default();
  unsafe { GetCursorPos(&mut point) }.ok().map(|()| point)
}

fn is_empty(rect: Rect) -> bool {
  !rect.valid() || rect.size.width <= 0.0 || rect.size.height <= 0.0
}

fn blend_state(
  device: &ID3D11Device,
  source_alpha: windows::Win32::Graphics::Direct3D11::D3D11_BLEND,
) -> Result<ID3D11BlendState, String> {
  let target = D3D11_RENDER_TARGET_BLEND_DESC {
    BlendEnable: true.into(),
    SrcBlend: D3D11_BLEND_SRC_ALPHA,
    DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOp: D3D11_BLEND_OP_ADD,
    SrcBlendAlpha: source_alpha,
    DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOpAlpha: D3D11_BLEND_OP_ADD,
    RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
  };
  let mut blend = None;
  unsafe {
    device.CreateBlendState(
      &D3D11_BLEND_DESC {
        RenderTarget: [target; 8],
        ..Default::default()
      },
      Some(&mut blend),
    )
  }
  .map_err(|error| error.to_string())?;
  blend.ok_or_else(|| "D3D11 created no region OSC blend state".to_owned())
}

fn sampler(
  device: &ID3D11Device,
  filter: windows::Win32::Graphics::Direct3D11::D3D11_FILTER,
) -> Result<ID3D11SamplerState, String> {
  let mut sampler = None;
  unsafe {
    device.CreateSamplerState(
      &D3D11_SAMPLER_DESC {
        Filter: filter,
        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
        MaxLOD: f32::MAX,
        ..Default::default()
      },
      Some(&mut sampler),
    )
  }
  .map_err(|error| error.to_string())?;
  sampler.ok_or_else(|| "D3D11 created no region OSC sampler".to_owned())
}

fn input_elements() -> [D3D11_INPUT_ELEMENT_DESC; 4] {
  [
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("POSITION"),
      SemanticIndex: 0,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 0,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 0,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 8,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 1,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 16,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 2,
      Format: DXGI_FORMAT_R32_UINT,
      InputSlot: 0,
      AlignedByteOffset: 24,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
  ]
}

/// The R8 icon atlas the shader reads for kinds 22-26. The pixels come from
/// the portable `screenwide_osc_icon_atlas`, which decodes the shared Lucide
/// sheet once; macOS cached one texture per `MTLDevice` and this caches one
/// per [`Gpu`], which is the same thing.
///
/// The atlas lives in a private module of the frozen `osc::controls` tree, so
/// it is reached through the `#[no_mangle]` export the macOS renderer already
/// used rather than by widening that module's visibility.
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeIconAtlas {
  pixels: *const u8,
  length: usize,
  width: u32,
  height: u32,
  columns: u32,
}

extern "C" {
  fn screenwide_osc_icon_atlas() -> NativeIconAtlas;
}

fn upload_icons(device: &ID3D11Device) -> Result<ID3D11ShaderResourceView, String> {
  let atlas = unsafe { screenwide_osc_icon_atlas() };
  let expected = atlas.width as usize * atlas.height as usize;
  if atlas.pixels.is_null() || atlas.length < expected || expected == 0 {
    return Err("the shared OSC icon atlas is empty".to_owned());
  }
  let pixels = unsafe { std::slice::from_raw_parts(atlas.pixels, expected) };
  upload_texture(
    device,
    pixels,
    atlas.width,
    atlas.height,
    DXGI_FORMAT_R8_UNORM,
    1,
  )
}

pub(super) fn upload_rgba(
  device: &ID3D11Device,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> Result<ID3D11ShaderResourceView, String> {
  upload_texture(device, rgba, width, height, DXGI_FORMAT_R8G8B8A8_UNORM, 4)
}

fn upload_texture(
  device: &ID3D11Device,
  rgba: &[u8],
  width: u32,
  height: u32,
  format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
  bytes_per_pixel: u32,
) -> Result<ID3D11ShaderResourceView, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let data = D3D11_SUBRESOURCE_DATA {
    pSysMem: rgba.as_ptr().cast::<c_void>(),
    SysMemPitch: width * bytes_per_pixel,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&data), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no region OSC texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  view.ok_or_else(|| "D3D11 created no region OSC texture view".to_owned())
}

/// Replaces `effectiveAppearance`: the shell's app theme preference.
fn light_mode() -> bool {
  let mut value = 0_u32;
  let mut length = size_of::<u32>() as u32;
  let status = unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
      w!("AppsUseLightTheme"),
      RRF_RT_REG_DWORD,
      None,
      Some((&raw mut value).cast::<c_void>()),
      Some(&mut length),
    )
  };
  status == ERROR_SUCCESS && value != 0
}

/// Creates a window on the thread that owns the host HWND: Win32 queues a
/// window's messages on its creating thread, so a surface window created on a
/// worker thread would never see a mouse message.
pub(crate) fn create_on_owning_thread(
  window: &tauri::WebviewWindow,
  host: HWND,
  peer: Option<(Rect, bool)>,
) -> Result<HWND, String> {
  struct HostHandle(HWND);
  unsafe impl Send for HostHandle {}
  struct Created(Result<HWND, String>);
  unsafe impl Send for Created {}

  let build = move |host: HWND| match peer {
    None => create_overlay(host),
    Some((bounds, capturable)) => create_peer(host, bounds, capturable),
  };
  if unsafe { GetWindowThreadProcessId(host, None) } == unsafe { GetCurrentThreadId() } {
    return build(host);
  }
  let handle = HostHandle(host);
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  window
    .run_on_main_thread(move || {
      let handle = handle;
      let _ = sender.send(Created(build(handle.0)));
    })
    .map_err(|error| format!("The Windows region OSC window could not be dispatched: {error}"))?;
  receiver
    .recv()
    .map_err(|_| "The Windows region OSC window was never created".to_owned())?
    .0
}

fn create_overlay(parent: HWND) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  let atom = *OVERLAY_CLASS.get_or_init(|| register_class(instance.0, w!("ScreenwideRegionOsc")));
  if atom == 0 {
    return Err("The Windows region OSC window class could not be registered".to_owned());
  }
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
      w!("ScreenwideRegionOsc"),
      PCWSTR::null(),
      WS_CHILD | WS_CLIPSIBLINGS,
      0,
      0,
      1,
      1,
      Some(parent),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows region OSC overlay could not be created: {error}"))?;
  Ok(hwnd)
}

/// The peer window: `NSWindowStyleMaskNonactivatingPanel` becomes
/// `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, and `NSWindowSharingNone` becomes
/// `WDA_EXCLUDEFROMCAPTURE` unless the user records Screenwide's own windows.
fn create_peer(owner: HWND, bounds: Rect, capturable: bool) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  let atom = *PEER_CLASS.get_or_init(|| register_class(instance.0, w!("ScreenwideRegionOscPeer")));
  if atom == 0 {
    return Err("The Windows region OSC peer class could not be registered".to_owned());
  }
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TRANSPARENT,
      w!("ScreenwideRegionOscPeer"),
      PCWSTR::null(),
      WS_POPUP,
      bounds.origin.x as i32,
      bounds.origin.y as i32,
      (bounds.size.width as i32).max(1),
      (bounds.size.height as i32).max(1),
      // Owned, not parented: the peer stays above the region selector without
      // becoming part of its client area, and closes with it.
      Some(owner),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows region OSC peer could not be created: {error}"))?;
  // Top-level cross-process click-through requires the peer to be both layered
  // and transparent. Full opacity keeps DirectComposition's own per-pixel
  // alpha intact while activating the layered-window hit-test behavior.
  unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), u8::MAX, LWA_ALPHA) }
    .map_err(|error| format!("The Windows region OSC peer could not enable layering: {error}"))?;
  disable_transitions(hwnd)?;
  if let Err(error) = set_capture_affinity(hwnd, capturable) {
    eprintln!("The Windows region OSC peer could not set capture affinity: {error}");
  }
  Ok(hwnd)
}

const fn peer_pointer_style(style: isize, passthrough: bool) -> isize {
  if passthrough {
    style | WS_EX_TRANSPARENT.0 as isize
  } else {
    style & !(WS_EX_TRANSPARENT.0 as isize)
  }
}

/// The Tauri host owns passthrough for the anchor child. An independent
/// top-level peer uses the documented layered-plus-transparent combination so
/// mouse targeting can continue into windows belonging to another process.
pub(crate) fn set_pointer_passthrough(hwnd: HWND, is_root: bool, passthrough: bool) -> bool {
  if is_root {
    return true;
  }
  let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let next = peer_pointer_style(current, passthrough);
  if next != current {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next) };
  }
  (unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) }) == next
}

fn disable_transitions(hwnd: HWND) -> Result<(), String> {
  let disabled = windows::core::BOOL(1);
  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_TRANSITIONS_FORCEDISABLED,
      (&raw const disabled).cast(),
      std::mem::size_of::<windows::core::BOOL>() as u32,
    )
  }
  .map_err(|error| error.to_string())
}

pub(crate) fn set_capture_affinity(hwnd: HWND, capturable: bool) -> Result<(), String> {
  let affinity = if capturable {
    WDA_NONE
  } else {
    WDA_EXCLUDEFROMCAPTURE
  };
  unsafe { SetWindowDisplayAffinity(hwnd, affinity) }.map_err(|error| error.to_string())
}

fn register_class(instance: *mut c_void, name: PCWSTR) -> u16 {
  unsafe {
    RegisterClassW(&WNDCLASSW {
      // Double clicks reach the controller as the modifier bit the region
      // gesture uses to expand to the full monitor.
      style: CS_DBLCLKS,
      lpfnWndProc: Some(input::window_proc),
      hInstance: HINSTANCE(instance),
      hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
      lpszClassName: name,
      ..Default::default()
    })
  }
}

#[cfg(test)]
#[path = "surface/tests.rs"]
mod tests;
