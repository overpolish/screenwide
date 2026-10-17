// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The workspace drawable: one swap chain over the whole viewport that the
//! shared scene draws every layer into, over its checkerboard, as the macOS
//! workspace layer is drawn. The panes are the layers' boxes, which
//! selection and hit-testing read; they draw nothing of their own.

use super::*;
use crate::app_windows::overlay_surface::light_mode;
use crate::editor::preview_platform::workspace_backdrop::BackdropStyle;
use crate::editor::preview_platform::workspace_scene::{NativeWorkspacePlacement, WorkspaceScene};
use screenwide_osc::gpu::magnifier::NativeMagnifier;

pub(super) struct WorkspaceView {
  pub(super) scene: WorkspaceScene,
  gpu: &'static screenwide_gpu::Gpu,
  drawable: Mutex<Drawable>,
  visual: IDCompositionVisual,
}

/// The swap chain and where its visual was last put, in physical pixels;
/// `None` while it is off screen.
struct Drawable {
  surface: Surface,
  offset: Option<(i32, i32)>,
}

/// What the workspace drawable shows, kept with the surface state.
#[derive(Default)]
pub(super) struct WorkspaceDraw {
  /// The pane each layer the scene holds is drawn in, in the scene's order.
  pub(super) panes: Vec<usize>,
  /// Whether the scene holds a recording's layers rather than a screenshot's.
  pub(super) recording: bool,
  pub(super) magnifier: Option<NativeMagnifier>,
  /// A draw an open present batch holds back for its flush.
  pub(super) pending: bool,
  /// A layout moved the panes ahead of the frame composed for them: the
  /// picture waits for that frame rather than show the last one in the new
  /// boxes.
  pub(super) held: bool,
}

impl WorkspaceView {
  pub(super) fn new(
    gpu: &'static screenwide_gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
    above: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows workspace visual could not be created: {error}"))?;
    unsafe {
      visual
        .SetOffsetX2(-100_000.0)
        .and_then(|_| root.AddVisual(&visual, true, Some(above)))
    }
    .map_err(|error| format!("The Windows workspace visual could not be attached: {error}"))?;
    let mut surface = Surface::on_visual(gpu, &visual)?;
    // The scene composes into a picture of its own and copies it in, so a
    // redraw of an unchanged scene costs one copy.
    surface.accept_copies(gpu);
    Ok(Self {
      scene: WorkspaceScene::new(cursor_artwork::native_cursors()?)?,
      gpu,
      drawable: Mutex::new(Drawable {
        surface,
        offset: None,
      }),
      visual,
    })
  }

  /// Takes the drawable off screen and gives its memory back. Answers whether
  /// it was on screen, which the caller has to commit.
  pub(super) fn hide(&self) -> bool {
    let Ok(mut drawable) = self.drawable.lock() else {
      return false;
    };
    if drawable.offset.take().is_none() {
      return false;
    }
    let _ = unsafe { self.visual.SetOffsetX2(-100_000.0) };
    drawable.surface.resize(self.gpu, (2, 2));
    true
  }

  /// Draws the scene into the drawable laid over `frame`, x, y, width and
  /// height in physical pixels. Answers whether the visual moved, which the
  /// caller has to commit.
  fn draw(
    &self,
    frame: (i32, i32, u32, u32),
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
  ) -> Result<bool, String> {
    let mut drawable = self
      .drawable
      .lock()
      .map_err(|_| "The Windows workspace drawable is poisoned".to_owned())?;
    let (x, y, width, height) = frame;
    drawable.surface.resize(self.gpu, (width, height));
    let moved = drawable.offset != Some((x, y));
    if moved {
      unsafe {
        self
          .visual
          .SetOffsetX2(x as f32)
          .and_then(|_| self.visual.SetOffsetY2(y as f32))
      }
      .map_err(|error| error.to_string())?;
      drawable.offset = Some((x, y));
    }
    // Nothing can be drawn now; the next change draws the scene again.
    let Frame::Ready(target) = drawable.surface.acquire(self.gpu)? else {
      return Ok(moved);
    };
    self
      .scene
      .draw(&target.texture, placements, magnifier, backdrop, true)?;
    self.gpu.queue.present(target);
    Ok(moved)
  }
}

/// Lets go of every layer the scene holds and takes the drawable away.
pub(super) fn clear_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  state.workspace = WorkspaceDraw::default();
  if let Err(error) = inner.gpu.workspace.scene.stage(&[]) {
    eprintln!("The editor workspace could not be cleared: {error}");
  }
  if inner.gpu.workspace.hide() {
    let _ = unsafe { inner.gpu.composition.Commit() };
  }
}

/// Draws the scene at the panes' current boxes. An open present batch holds
/// the draw back for its flush, so the layout and the picture land together;
/// a workspace with no layers shows the backstop alone, as macOS does.
pub(super) fn draw_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  if inner.batch_depth.load(Ordering::Acquire) > 0 {
    state.workspace.pending = true;
    return;
  }
  state.workspace.pending = false;
  if state.workspace.held {
    return;
  }
  let viewport = state.viewport;
  if state.workspace.panes.is_empty() || viewport.width < 1.0 || viewport.height < 1.0 {
    if inner.gpu.workspace.hide() {
      let _ = unsafe { inner.gpu.composition.Commit() };
    }
    return;
  }
  let (x, right) = window::scaled_edges(viewport.x, viewport.width, state.scale);
  let (y, bottom) = window::scaled_edges(viewport.y, viewport.height, state.scale);
  let placements = state
    .workspace
    .panes
    .iter()
    .map(|index| {
      state
        .panes
        .get(*index)
        .and_then(Option::as_ref)
        .filter(|pane| pane.seen)
        .map_or_else(NativeWorkspacePlacement::default, |pane| {
          placement(pane, (x, y))
        })
    })
    .collect::<Vec<_>>();
  let backdrop = BackdropStyle {
    scale: state.scale,
    light: light_mode(),
  };
  let frame = (x, y, (right - x).max(1) as u32, (bottom - y).max(1) as u32);
  match inner.gpu.workspace.draw(
    frame,
    &placements,
    state.workspace.magnifier.as_ref(),
    backdrop,
  ) {
    Ok(true) => {
      let _ = unsafe { inner.gpu.composition.Commit() };
    }
    Ok(false) => {}
    Err(error) => eprintln!("The editor workspace could not be drawn: {error}"),
  }
}

/// Draws the scene a present just staged, whatever a layout held back.
pub(super) fn present_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  state.workspace.held = false;
  draw_workspace(inner, state);
}

/// Where `pane` puts its canvas in the drawable whose corner is `origin`: its
/// box, with the canvas centred in it where their aspects differ for the
/// moment a resized canvas takes to arrive.
fn placement(pane: &Pane, origin: (i32, i32)) -> NativeWorkspacePlacement {
  let content = (
    f64::from(pane.content_size.0.max(1)),
    f64::from(pane.content_size.1.max(1)),
  );
  let display = (
    f64::from(pane.display_size.0.max(1)),
    f64::from(pane.display_size.1.max(1)),
  );
  let scale = (display.0 / content.0).min(display.1 / content.1);
  let width = (content.0 * scale).round();
  let height = (content.1 * scale).round();
  NativeWorkspacePlacement {
    x: (f64::from(pane.position.0 - origin.0) + (display.0 - width) / 2.0).round() as i32,
    y: (f64::from(pane.position.1 - origin.1) + (display.1 - height) / 2.0).round() as i32,
    width: width as u32,
    height: height as u32,
  }
}
