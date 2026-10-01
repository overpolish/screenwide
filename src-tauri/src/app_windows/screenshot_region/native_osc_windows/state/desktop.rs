// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// True when this binding describes a different desktop than the last one.
pub(crate) fn layout_changed(previous: Option<&LayoutSnapshot>, binding: &DesktopBinding) -> bool {
  previous
    .is_none_or(|(displays, anchor)| *anchor != binding.anchor_id || *displays != binding.displays)
}

/// Enumerates the desktop and stamps `layout_changed` from the last binding
/// this context configured.
pub(crate) fn configure_desktop_window(
  window: &WebviewWindow,
  anchor_id: u32,
) -> Result<DesktopBinding, String> {
  let (mut binding, probes) = super::super::desktop::configure_window(window, anchor_id)?;
  if binding.anchor_id != anchor_id {
    eprintln!(
      "The Windows region OSC substituted monitor {} for the lost anchor {anchor_id}",
      binding.anchor_id
    );
  }
  let changed = with_context(window, |context| {
    if let Ok(mut stored) = context.probes.lock() {
      *stored = probes;
    }
    let Ok(mut layout) = context.layout.lock() else {
      return true;
    };
    let established = layout.is_some();
    let changed = layout_changed(layout.as_ref(), &binding);
    if changed {
      *layout = Some((binding.displays.clone(), binding.anchor_id));
    }
    if let Ok(mut pending) = context.pending_layout_notice.lock() {
      *pending = changed && established;
    }
    changed
  })
  .unwrap_or(true);
  binding.layout_changed = changed;
  Ok(binding)
}

/// Port of `configure_desktop` (`native_osc_macos/state.rs:598`): the desktop
/// union replaces the single monitor as the controller's coordinate space, the
/// anchor's origin becomes the root surface's desktop offset, and the peer
/// windows are rebuilt whenever the topology moved.
pub(crate) fn configure_desktop(
  window: &WebviewWindow,
  binding: DesktopBinding,
  local: Option<Rect>,
) -> bool {
  let Some(configured) = with_context(window, |context| {
    let Some(anchor) = binding.anchor() else {
      return false;
    };
    let committed = global_committed(&binding, local);
    let controller = RegionController::new(binding.virtual_monitor(), committed, None);
    let Ok(mut current_controller) = context.runtime.controller.lock() else {
      return false;
    };
    let Ok(mut desktop) = context.runtime.desktop.lock() else {
      return false;
    };
    if let Ok(mut set) = context.surfaces.lock() {
      let root = set.root_mut();
      root.display_id = binding.anchor_id;
      root.set_desktop_offset(anchor.origin);
    }
    *current_controller = controller;
    *desktop = Some(binding.clone());
    true
  }) else {
    return false;
  };
  if !configured {
    return false;
  }
  sync_peers(window, &binding);
  // The peers exist before the webview hears about the new topology.
  let notify = with_context(window, |context| {
    context
      .pending_layout_notice
      .lock()
      .map(|mut pending| std::mem::replace(&mut *pending, false))
      .unwrap_or(false)
  })
  .unwrap_or(false);
  if notify {
    with_context(window, notify_layout_changed);
  }
  true
}

/// Port of `rebuild_surfaces` (`+desktop.m:113-186`): a topology change tears
/// the peers down - cancelling their gestures, lens and cursor first - and
/// rebuilds them from the new binding.
pub(super) fn sync_peers(window: &WebviewWindow, binding: &DesktopBinding) {
  let Some(context) = context_arc(window) else {
    return;
  };
  let probes = context
    .probes
    .lock()
    .map(|probes| probes.clone())
    .unwrap_or_default();
  let plan = peer_plan(binding, &probes);
  {
    let Ok(set) = context.surfaces.lock() else {
      return;
    };
    if set.peers_match(&plan) {
      return;
    }
  }
  let capturable = crate::settings::current(window.app_handle()).record_screenwide_windows;
  // Windows must be created on the thread that owns the host, and that thread
  // must not be blocked while a surface lock is held.
  let mut built = Vec::new();
  for planned in &plan {
    let peer =
      surface::create_on_owning_thread(window, context.host, Some((planned.bounds, capturable)))
        .and_then(|hwnd| {
          Surface::peer(
            Arc::clone(&context.gpu),
            hwnd,
            planned.display_id,
            planned.bounds,
            planned.scale,
          )
        });
    match peer {
      Ok(peer) => built.push((peer, planned.offset)),
      Err(error) => {
        eprintln!("The Windows region OSC peer could not be created: {error}");
        return;
      }
    }
  }
  // The registry is only ever touched outside the surface lock, so the two
  // never nest in opposite orders.
  let (retired, adopted) = {
    let Ok(mut set) = context.surfaces.lock() else {
      return;
    };
    let retired = set
      .peers
      .drain(..)
      .map(|peer| peer.hwnd())
      .collect::<Vec<_>>();
    // The rebuild cancels whatever the old peers were doing before the new
    // ones inherit the scene.
    let root = set.root_mut();
    root.release_pointer();
    root.gesture_active = false;
    root.magnifier = None;
    let region = root.region;
    let visible = root.visible;
    let show_frame = root.show_frame;
    let show_handles = root.show_handles;
    let input_enabled = root.input_enabled;
    let desktop_presented = root.desktop_presented;
    let snapshot_presented = root.snapshot_presented;
    let snapshot_composited = root.snapshot_composited;
    let mut adopted = Vec::with_capacity(built.len());
    for (mut peer, offset) in built {
      peer.set_desktop_offset(offset);
      peer.show_frame = show_frame;
      peer.show_handles = show_handles;
      peer.input_enabled = input_enabled;
      peer.desktop_presented = desktop_presented;
      peer.snapshot_presented = snapshot_presented;
      peer.snapshot_composited = snapshot_composited;
      adopted.push((peer.hwnd(), input_enabled));
      peer.set_region(region, visible);
      set.peers.push(peer);
    }
    (retired, adopted)
  };
  for hwnd in retired {
    unregister_surface(hwnd);
  }
  for (hwnd, input_enabled) in adopted {
    register_surface(hwnd, &context);
    surface::set_pointer_passthrough(hwnd, false, !input_enabled);
  }
}

/// Port of `native_osc_layout_changed` (`native_osc_macos/state.rs:362-382`).
/// Region hosts tell their webview; the other purposes restart their sessions
/// in Rust once stages 3 and 4 land.
pub(crate) fn notify_layout_changed(context: &Context) {
  {
    let Ok(mut last) = context.layout_notified.lock() else {
      return;
    };
    let now = Instant::now();
    if last.is_some_and(|last| now.duration_since(last) < LAYOUT_COALESCE) {
      return;
    }
    *last = Some(now);
  }
  match context.runtime.purpose {
    Purpose::Region => {
      let window = &context.runtime.window;
      let _ = window.emit_to(
        EventTarget::webview_window(window.label()),
        super::super::NATIVE_OSC_LAYOUT_EVENT,
        (),
      );
    }
    Purpose::TextRecognition => {
      crate::text_recognition::restart_after_topology_change(context.runtime.window.app_handle());
    }
    Purpose::Ruler => {
      crate::ruler::restart_after_topology_change(context.runtime.window.app_handle());
    }
  }
}

/// Called from a peer's window procedure on `WM_DISPLAYCHANGE`/`WM_DPICHANGED`.
pub(crate) fn notify_layout_changed_for_surface(hwnd: HWND) {
  if let Some(context) = context_for_surface(hwnd) {
    notify_layout_changed(&context);
  }
}
