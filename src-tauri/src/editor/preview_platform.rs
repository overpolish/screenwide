// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform facade for the editor window's live preview.
//!
//! # What this facade is
//!
//! The editor window previews a recording or a screenshot by compositing frames
//! on the GPU into panes that sit *below* the OS webview. The webview draws only
//! UI chrome - on-screen controls, backdrop layers with CSS mask holes - so the
//! pixels the user is editing never cross IPC.
//!
//! Everything platform-specific about that lives behind this module and behind
//! [`super::recording_preview_player::platform`]. Shared Rust above the facade
//! owns geometry, layout and settings math (`recording_preview_player::layout`,
//! `media_preview::bake`, output validation) and must never assume a particular
//! GPU API.
//!
//! # Porting to a new platform
//!
//! A backend is a module that exports the same item names with the same
//! signatures as `surface_windows` and `surface_macos`, selected by the
//! `cfg` block below. Both are thin shells over the shared editing engine
//! ([`screenwide_workspace::editing::WorkspaceEditor`]): a shell implements its
//! `Host` trait and hands the engine its pointer and keyboard. Five pieces
//! make up a complete port, and a port has to be complete: the frontend has no
//! software preview to fall back to, so a backend that omits any of them
//! leaves that platform without a preview at all.
//!
//! 1. **A compositing surface created from a tauri window, rendering below the
//!    OS webview.** [`RecordingPreviewSurface::from_window`] takes the editor
//!    [`WebviewWindow`] and attaches native drawables underneath the webview's
//!    own view, so webview chrome composites on top of them, and an input
//!    surface over the workarea. On macOS those are `CAMetalLayer`s on a view
//!    inserted below the `WKWebView`, with an input `NSView` above it; on
//!    Windows a DirectComposition visual tree under the `WebView2` child and a
//!    non-activating input child window above it.
//! 2. **The workspace drawable.** Both backends draw every layer through the
//!    shared [`screenwide_workspace::scene`]: `present_screenshot_workspace`
//!    and `present_recording_workspace` stage the layers, each from a
//!    platform picture (`LayerPicture`), and the surface draws the scene into
//!    one drawable over the viewport, at its panes' boxes and over the
//!    checkerboard, whenever a layout, a transform or a present changes it.
//!    The scene does the whole output composition on the GPU. The backstop,
//!    the workspace and the chrome are clipped to the viewport's rounded
//!    corners, which the webview's hole over them shares.
//! 3. **A pane decoder and frame scrubber for stills and scrubbing**, plus (4)
//!    the **playback video path** - both live in
//!    [`super::recording_preview_player::platform`], which selects backends the
//!    same way this module does. The decoder answers "give me the frame at t,
//!    sized for this pane"; the scrubber keeps a warm long-lived decode context
//!    so dragging the timeline does not reopen a GOP per frame.
//! 5. **The export compositor** (writing the final file rather than the screen)
//!    is already split per platform in [`super::cursor_export`], under
//!    `platform_macos` / `platform_unsupported`.
//!
//! # Platform GPU backends
//!
//! Shaders are WGSL on the shared wgpu device ([`screenwide_gpu`]), drawn by the
//! shared [`compositor`]; the live preview's workspace is the shared
//! [`screenwide_workspace::scene`] on both platforms. Windows presents through
//! DirectComposition and macOS through Core Animation layers. Another OS can
//! add its own backend behind this facade without leaking platform texture
//! formats, coordinate conventions, or window-surface details into shared
//! preview code.

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod audio_ribbon;

#[cfg(target_os = "macos")]
mod surface_macos;
#[cfg(target_os = "macos")]
use self::surface_macos as surface;
#[cfg(target_os = "windows")]
mod surface_windows;
#[cfg(target_os = "windows")]
use self::surface_windows as surface;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use audio_ribbon::{ribbon_still, BAR_PITCH_POINTS as RIBBON_BAR_PITCH_POINTS};
pub(crate) use surface::RecordingPreviewSurface;

pub(crate) const NATIVE_POINTER_EVENT: &str = "preview://native-pointer";

/// A press on the picture as the editor window hears of it: `down` as it
/// goes down, `took` when it took hold of something already there, and
/// `down: false` as it is let go.
#[derive(Clone, Copy, serde::Serialize)]
struct NativePointerEvent {
  down: bool,
  took: bool,
}

pub(crate) const POINTER_ARRIVED_EVENT: &str = "preview://pointer-arrived";

/// Tells `window` of every press on its picture, which blurs the page's
/// focus and lets the editor know a press is under way, and of the pointer
/// coming onto the picture, which the page hears of no other way.
pub(crate) fn pointer_press_callback(
  window: tauri::WebviewWindow,
) -> screenwide_workspace::editing::PointerPressCallback {
  use screenwide_workspace::editing::PointerPress;
  use tauri::Emitter;
  Box::new(move |press| {
    let event = match press {
      PointerPress::Down { took } => NativePointerEvent { down: true, took },
      PointerPress::Up => NativePointerEvent {
        down: false,
        took: false,
      },
      PointerPress::Arrived => {
        let _ = window.emit(POINTER_ARRIVED_EVENT, ());
        return;
      }
    };
    let _ = window.emit(NATIVE_POINTER_EVENT, event);
  })
}

pub(crate) const NATIVE_CONTEXT_MENU_EVENT: &str = "preview://context-menu";

/// A picture dropped on the workspace, and where it landed: the layer under
/// it and the point in that layer's image-normalised space, or `None` off
/// every picture.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) type ImageDropCallback = Box<
  dyn FnMut(
      crate::editor::images::dropped::DroppedPicture,
      Option<crate::editor::images::dropped::ImagePoint>,
    ) + Send
    + 'static,
>;

/// Builds the comparatively expensive GPU pipeline while the editor webview is
/// still hidden. The first recording review can then open with only its media
/// source to initialise.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn prewarm(window: tauri::WebviewWindow) {
  tauri::async_runtime::spawn_blocking(move || {
    if let Err(error) = RecordingPreviewSurface::from_window(&window) {
      eprintln!("Could not prewarm the preview surface: {error}");
    }
  });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn prewarm(_window: tauri::WebviewWindow) {}
