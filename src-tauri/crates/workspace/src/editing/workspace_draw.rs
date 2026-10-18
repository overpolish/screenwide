// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the workspace is drawn and where each layer lands in it. The host
//! owns the drawable; the panes are the layers' boxes, which selection and
//! hit-testing read, and draw nothing of their own.

use super::*;
use crate::backdrop::BackdropStyle;
use crate::scene::NativeWorkspacePlacement;
use screenwide_osc::gpu::magnifier::NativeMagnifier;

/// What the workspace drawable shows, kept with the surface state.
#[derive(Default)]
pub struct WorkspaceDraw {
  /// The pane each layer the scene holds is drawn in, in the scene's order.
  pub panes: Vec<usize>,
  /// Whether the scene holds a recording's layers rather than a screenshot's.
  pub recording: bool,
  pub magnifier: Option<NativeMagnifier>,
  /// A draw an open present batch holds back for its flush.
  pub pending: bool,
  /// A layout moved the panes ahead of the frame composed for them: the
  /// picture waits for that frame rather than show the last one in the new
  /// boxes.
  pub held: bool,
  /// A layout is under way: its panes are not all placed yet, and a draw
  /// now would leave out the ones still to come. Its finish draws.
  pub laying_out: bool,
}

/// Lets go of every layer the scene holds and takes the drawable away.
pub fn clear_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  state.workspace = WorkspaceDraw::default();
  if let Err(error) = inner.host.scene().stage(&[]) {
    eprintln!("The editor workspace could not be cleared: {error}");
  }
  if inner.host.hide_workspace() {
    inner.host.commit(false);
  }
}

/// Draws the scene at the panes' current boxes. An open present batch holds
/// the draw back for its flush, so the layout and the picture land together;
/// a workspace with no layers shows the backstop alone, as macOS does.
pub fn draw_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  if inner.batch_depth.load(Ordering::Acquire) > 0 {
    state.workspace.pending = true;
    return;
  }
  state.workspace.pending = false;
  if state.workspace.laying_out {
    return;
  }
  if state.workspace.held {
    return;
  }
  let viewport = state.viewport;
  if state.workspace.panes.is_empty() || viewport.width < 1.0 || viewport.height < 1.0 {
    if inner.host.hide_workspace() {
      inner.host.commit(false);
    }
    return;
  }
  let (x, right) = pixels::scaled_edges(viewport.x, viewport.width, state.scale);
  let (y, bottom) = pixels::scaled_edges(viewport.y, viewport.height, state.scale);
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
  let light = inner.host.light_mode();
  let backdrop = BackdropStyle {
    scale: state.scale,
    light,
  };
  let magnifier = state.workspace.magnifier.map(|magnifier| NativeMagnifier {
    light_mode: u32::from(light),
    ..magnifier
  });
  let frame = (x, y, (right - x).max(1) as u32, (bottom - y).max(1) as u32);
  match inner
    .host
    .draw_workspace(frame, &placements, magnifier.as_ref(), backdrop)
  {
    Ok(true) => inner.host.commit(false),
    Ok(false) => {}
    Err(error) => eprintln!("The editor workspace could not be drawn: {error}"),
  }
}

/// Draws the scene a present just staged, whatever a layout held back.
pub fn present_workspace(inner: &SurfaceInner, state: &mut SurfaceState) {
  state.workspace.held = false;
  draw_workspace(inner, state);
}

/// Where `pane` puts its canvas in the drawable whose corner is `origin`: its
/// box, with the canvas centred in it where their aspects differ for the
/// moment a resized canvas takes to arrive. The corner lands on a whole
/// pixel and the size is the canvas at one scale both ways: rounded
/// separately, the two sides scale apart by up to a pixel, and which way
/// each rounds changes from one zoom step to the next, so the picture
/// shimmers as it zooms.
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
  let width = content.0 * scale;
  let height = content.1 * scale;
  NativeWorkspacePlacement {
    x: (f64::from(pane.position.0 - origin.0) + (display.0 - width) / 2.0).round() as i32,
    y: (f64::from(pane.position.1 - origin.1) + (display.1 - height) / 2.0).round() as i32,
    width: width as f32,
    height: height as f32,
  }
}
