// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one display draws this frame.

use super::*;

/// Everything on screen plus the stroke in hand, prepared in this display's
/// layer pixels.
///
/// The annotations are carried into the display's pixels first, so the arrows
/// are prepared through the identity placement: the editor's own preparation,
/// with no canvas to fit them to.
pub(super) fn scene(display: Display) -> arrows::PreparedArrows {
  let drawn: Vec<_> = live_clips::annotations()
    .iter()
    .chain(input::in_progress().iter())
    .map(|annotation| geometry::display_annotation(annotation, display.origin, display.scale))
    .collect();
  arrows::placed_arrows(&drawn, (0.0, 0.0), (1.0, 1.0), None, None)
}
