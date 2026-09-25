// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a redaction on a recording reads ahead of drawing it: a secure
//! pixelation's zones, once from its clip's first frame, and the surface
//! round the box across the whole clip.
//!
//! A secure pixelation takes each zone's colours from under it. Read again
//! on every frame, they would follow whatever moves under the box: text
//! scrolling a pixel a frame would show, frame by frame, where its ink sits
//! to the pixel. Held from one frame, they show no more than a screenshot of
//! that frame would. The surface is read from the ring outside the box,
//! which shows nothing the box covers, so it follows the background: worked
//! out for the whole clip at once, so a change lands where it happened and
//! something busy passing by is not followed.

/// A held fill: the surface around the box, and a secure pixelation's zones
/// as [`super::palette::zones`] reads them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HeldFill {
  /// The surface around the box, `None` where nothing opaque surrounds it.
  pub(crate) surface: Option<[u8; 3]>,
  /// Zones across, and blocks a zone's side; zero for every other mode.
  pub(crate) columns: u32,
  pub(crate) blocks: u32,
  /// Each zone's two packed inks, in rows from the top-left.
  pub(crate) inks: Vec<[f32; 2]>,
  /// How the surface changes over the clip, as `surface_timeline` works it
  /// out ahead from the ring round the box; empty where it holds one colour
  /// or where the frame in hand has already had it resolved.
  pub(crate) surfaces: Vec<[f32; 2]>,
}
