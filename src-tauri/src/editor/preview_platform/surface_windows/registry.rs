// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// One compositor per editor window: a DirectComposition target belongs to the
/// host HWND it was created for, so a process-wide surface would make the
/// second editor window composite into the first one's window.
///
/// The per-window slot is an inner `OnceLock` handed out from under the map
/// lock rather than a `Result` stored in the map, because creating a surface
/// round-trips to the window's event-loop thread
/// ([`create_editor_on_owning_thread`]) and must not run while a lock that
/// thread could also want is held. The slot keeps the original semantics:
/// created at most once per window, and a failure cached so a broken GPU is not
/// retried forever.
pub(super) type SurfaceSlot =
  std::sync::Arc<OnceLock<Result<std::sync::Arc<SurfaceInner>, String>>>;

static PREVIEW_SURFACES: OnceLock<Mutex<HashMap<isize, SurfaceSlot>>> = OnceLock::new();

/// Reverse lookups, filled once a surface exists. The editor `window_proc` is
/// handed only its own HWND, and the recording export knows only which
/// workspace it is saving; neither can name the host window.
#[derive(Default)]
pub(super) struct SurfaceIndex {
  pub(super) by_editor: HashMap<isize, std::sync::Arc<SurfaceInner>>,
  pub(super) by_kind: HashMap<crate::editor::EditorKind, std::sync::Arc<SurfaceInner>>,
}

static SURFACE_INDEX: OnceLock<Mutex<SurfaceIndex>> = OnceLock::new();

pub(super) fn preview_surfaces() -> &'static Mutex<HashMap<isize, SurfaceSlot>> {
  PREVIEW_SURFACES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn surface_index() -> &'static Mutex<SurfaceIndex> {
  SURFACE_INDEX.get_or_init(|| Mutex::new(SurfaceIndex::default()))
}

pub(super) fn surface_for_editor(hwnd: HWND) -> Option<std::sync::Arc<SurfaceInner>> {
  let index = surface_index().lock().ok()?;
  index
    .by_editor
    .get(&(hwnd.0 as isize))
    .map(std::sync::Arc::clone)
}
