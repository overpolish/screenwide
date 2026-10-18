// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) struct Gpu {
  pub(super) audio_ribbon: std::sync::Mutex<audio_ribbon::AudioRibbon>,
  pub(super) backdrop: Backdrop,
  /// Direct3D 11 on the shared device: Media Foundation decodes on it.
  pub(super) d3d11: &'static D3d11Layer,
  pub(super) composition: IDCompositionDevice,
  pub(super) selection: Mutex<selection::SelectionOverlay>,
  pub(super) _editor_target: IDCompositionTarget,
  pub(super) _target: IDCompositionTarget,
  pub(super) workspace: WorkspaceView,
}
