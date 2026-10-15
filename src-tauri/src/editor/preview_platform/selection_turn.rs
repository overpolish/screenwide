// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::PreviewSelection;
use screenwide_scenes::SceneRotation;

impl PreviewSelection {
  /// This selection as one the select tool turns with the rotation gizmo,
  /// turned as `rotation` has it now; `None` leaves it a layer nothing turns.
  pub(crate) fn turned_by(self, rotation: Option<SceneRotation>) -> Self {
    Self {
      turnable: u32::from(rotation.is_some()),
      rotation_x: rotation.map_or(0.0, |rotation| rotation.x),
      rotation_y: rotation.map_or(0.0, |rotation| rotation.y),
      rotation_z: rotation.map_or(0.0, |rotation| rotation.z),
      ..self
    }
  }
}
