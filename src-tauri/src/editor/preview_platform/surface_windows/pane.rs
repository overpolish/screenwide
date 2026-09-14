// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Backdrop {
  pub(super) fn new(
    composition: &IDCompositionDevice,
    factory: &IDXGIFactory2,
    device: &ID3D11Device,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let description = DXGI_SWAP_CHAIN_DESC1 {
      Width: 2,
      Height: 2,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      Stereo: false.into(),
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
      AlphaMode: windows::Win32::Graphics::Dxgi::Common::DXGI_ALPHA_MODE_PREMULTIPLIED,
      Flags: 0,
    };
    let swap_chain = unsafe { factory.CreateSwapChainForComposition(device, &description, None) }
      .and_then(|chain| chain.cast::<IDXGISwapChain3>())
      .map_err(|error| format!("The Windows preview backstop could not be created: {error}"))?;
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| {
      format!("The Windows preview backstop visual could not be created: {error}")
    })?;
    let scale_transform = unsafe { composition.CreateScaleTransform() }.map_err(|error| {
      format!("The Windows preview backstop transform could not be created: {error}")
    })?;
    (|| -> windows::core::Result<()> {
      unsafe {
        visual.SetContent(&swap_chain)?;
        visual.SetTransform(&scale_transform)?;
        visual.SetOffsetX2(-100_000.0)?;
        // This is the native equivalent of macOS's opaque container layer: it
        // sits below every video pane but fills the complete preview viewport.
        root.AddVisual(&visual, false, None::<&IDCompositionVisual>)?;
      }
      Ok(())
    })()
    .map_err(|error| format!("The Windows preview backstop could not be attached: {error}"))?;
    Ok(Self {
      scale_transform,
      swap_chain,
      visual,
    })
  }

  pub(super) fn paint(
    &self,
    context: &ID3D11DeviceContext,
    colour: [f64; 4],
  ) -> Result<(), String> {
    let target = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }
      .map_err(|error| format!("The Windows preview backstop has no buffer: {error}"))?;
    let resource: ID3D11Resource = target.cast().map_err(|error| error.to_string())?;
    let device = unsafe { target.GetDevice() }.map_err(|error| error.to_string())?;
    let mut view: Option<ID3D11RenderTargetView> = None;
    unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut view)) }
      .map_err(|error| format!("The Windows preview backstop could not be painted: {error}"))?;
    let view = view.ok_or_else(|| "D3D11 created no preview backstop view".to_owned())?;
    let alpha = colour[3].clamp(0.0, 1.0) as f32;
    let colour = [
      colour[0].clamp(0.0, 1.0) as f32 * alpha,
      colour[1].clamp(0.0, 1.0) as f32 * alpha,
      colour[2].clamp(0.0, 1.0) as f32 * alpha,
      alpha,
    ];
    unsafe { context.ClearRenderTargetView(&view, &colour) };
    unsafe { self.swap_chain.Present(0, DXGI_PRESENT(0)) }
      .ok()
      .map_err(|error| format!("The Windows preview backstop could not be presented: {error}"))
  }

  pub(super) fn set_geometry(&self, rect: PreviewSurfaceRect, scale: f64) {
    if rect.width < 1.0 || rect.height < 1.0 {
      self.hide();
      return;
    }
    let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
    let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
    let _ = unsafe {
      self
        .visual
        .SetOffsetX2(x as f32)
        .and_then(|_| self.visual.SetOffsetY2(y as f32))
        .and_then(|_| {
          self
            .scale_transform
            .SetScaleX2((right - x).max(2) as f32 / 2.0)
        })
        .and_then(|_| {
          self
            .scale_transform
            .SetScaleY2((bottom - y).max(2) as f32 / 2.0)
        })
    };
  }

  pub(super) fn hide(&self) {
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
  }
}

impl Pane {
  pub(super) fn release_drawables(&mut self, context: &ID3D11DeviceContext) -> Result<(), String> {
    self.blur = None;
    unsafe {
      let vertex_buffer: Option<ID3D11Buffer> = None;
      let stride = 0_u32;
      let offset = 0_u32;
      context.IASetVertexBuffers(
        0,
        1,
        Some(&raw const vertex_buffer),
        Some(&raw const stride),
        Some(&raw const offset),
      );
      context.PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      context.OMSetRenderTargets(None, None);
      self
        .swap_chain
        .ResizeBuffers(2, 2, 2, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SWAP_CHAIN_FLAG(0))
    }
    .map_err(|error| format!("The Windows preview pane could not release buffers: {error}"))
    .inspect(|()| self.buffer_size = (2, 2))
  }

  pub(super) fn update_geometry(&self) -> windows::core::Result<()> {
    let content_width = self.content_size.0.max(1) as f32;
    let content_height = self.content_size.1.max(1) as f32;
    let display_width = self.display_size.0.max(1) as f32;
    let display_height = self.display_size.1.max(1) as f32;
    let (clip_left, clip_top, clip_right, clip_bottom) = self.clip_edges;
    // One uniform scale, so a composed canvas whose aspect no longer matches
    // the laid-out box is centred inside it rather than stretched across it.
    // Aspects agree in the steady state and this is then exactly the
    // per-axis mapping; it only bites in the window between a re-composed
    // canvas and the layout that catches up with it.
    let scale = (display_width / content_width).min(display_height / content_height);
    let inset_x = (display_width - content_width * scale) / 2.0;
    let inset_y = (display_height - content_height * scale) / 2.0;
    unsafe {
      self.visual.SetOffsetX2(self.position.0 as f32 + inset_x)?;
      self.visual.SetOffsetY2(self.position.1 as f32 + inset_y)?;
      self.scale_transform.SetScaleX2(scale)?;
      self.scale_transform.SetScaleY2(scale)?;
      self.clip.SetLeft2((clip_left as f32 - inset_x) / scale)?;
      self.clip.SetTop2((clip_top as f32 - inset_y) / scale)?;
      self.clip.SetRight2((clip_right as f32 - inset_x) / scale)?;
      self
        .clip
        .SetBottom2((clip_bottom as f32 - inset_y) / scale)?;
    }
    Ok(())
  }

  pub(super) fn hide(&self) {
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
  }
}
