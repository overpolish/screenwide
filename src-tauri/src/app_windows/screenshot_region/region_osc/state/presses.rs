// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Presses taken from the region engine for something else. While they are
//! taken, every surface of the overlay takes a press itself, whatever its
//! input says, and hands it on instead of letting it through to the window
//! under it. Window picking takes them while its input is off, so they reach
//! neither the region engine nor the windows behind the frozen desktop.

use super::*;

type Taker = Arc<dyn Fn() + Send + Sync>;

/// The overlay whose presses are taken, by its host window, and what takes
/// them. Held apart from the context so it also covers an overlay that
/// attaches after the presses were taken. Never held with another lock.
static TAKEN: Mutex<Option<(WindowId, Taker)>> = Mutex::new(None);

fn taken() -> std::sync::MutexGuard<'static, Option<(WindowId, Taker)>> {
  TAKEN
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Hands the presses on `window` to `taker`, or with none gives them back.
/// Only Windows takes them; macOS catches a pick's press before the overlay.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn take_presses(window: &WebviewWindow, taker: Option<Taker>) {
  let Some(host_window) = attachment::host(window) else {
    return;
  };
  *taken() = taker.map(|taker| (host_window, taker));
  let Some((context, peers)) = context_arc(window).and_then(|context| {
    let peers = context
      .surfaces
      .lock()
      .ok()?
      .peers
      .iter()
      .map(|peer| (peer.hwnd(), peer.input_enabled))
      .collect::<Vec<_>>();
    Some((context, peers))
  }) else {
    return;
  };
  // The anchor surface needs no style: it answers each hit test itself.
  for (hwnd, input_enabled) in peers {
    host().set_pointer_passthrough(hwnd, false, surface_passthrough(&context, input_enabled));
  }
}

pub(crate) fn presses_taken(context: &Context) -> bool {
  taken()
    .as_ref()
    .is_some_and(|(host_window, _)| *host_window == context.host)
}

/// Whether a surface lets presses through to the window under it.
pub(super) fn surface_passthrough(context: &Context, input_enabled: bool) -> bool {
  !input_enabled && !presses_taken(context)
}

/// What takes a press on `surface`, when its overlay's presses are taken.
#[cfg(target_os = "windows")]
pub(crate) fn press_taker(surface: WindowId) -> Option<Taker> {
  let context = context_for_surface(surface)?;
  taken()
    .as_ref()
    .filter(|(host_window, _)| *host_window == context.host)
    .map(|(_, taker)| Arc::clone(taker))
}
