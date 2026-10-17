// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct NativeWorkspacePaneRect {
  pub(super) index: u32,
  pub(super) x: f64,
  pub(super) y: f64,
  pub(super) width: f64,
  pub(super) height: f64,
}
