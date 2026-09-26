// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pinning: an annotation on a recording following the content it was
//! placed on.
//!
//! The tracker reads nothing but brightness, decoded straight to a reduced
//! tracking size by the platform decoder, so the only full-size work is the
//! decode the hardware already does. From each keyframe it follows a cloud of
//! corner points in both directions: each point is carried frame to frame by
//! pyramidal Lucas-Kanade and kept only if it tracks back to where it came
//! from, and the points agree on one movement and change of size by RANSAC.
//! Where the keyframe's own points can still be found, the movement is
//! measured against them instead, which is what keeps a long scroll from
//! drifting. A target that leaves the frame or stops matching is searched for
//! again near where it was last seen.
//!
//! The legs between keyframes are joined and smoothed forward and backward
//! with a constant velocity Kalman model whose trust in each frame follows
//! how well that frame matched: a clean match on screen content passes
//! through untouched, and a noisy one on moving footage is steadied.

/// Joining legs into one path.
mod assemble;
/// Corner points worth following inside a region.
mod features;
/// The movement and change of size a set of point pairs agree on.
mod fit;
/// Pyramidal Lucas-Kanade for one point.
mod flow;
/// Boxes, similarities and a deterministic generator.
mod geometry;
/// Following the target from one keyframe.
pub(crate) mod leg;
/// A frame's brightness at the tracking size, and its pyramid.
pub(crate) mod luma;
/// The pin as the document stores it.
pub(crate) mod model;
/// A path shown while it is still being worked out.
pub(crate) mod partial;
/// Planning and running a pin's legs.
pub(crate) mod path;
/// Where a pinned annotation is drawn, and how an edit is written back.
pub(crate) mod resolve;
/// Finding a lost target again near where it was last seen.
mod search;
/// Forward-backward Kalman smoothing of a raw path.
mod smooth;
/// What a pinned annotation follows.
pub(crate) mod target;
/// One direction's frame-by-frame state.
mod tracker;

pub(crate) use assemble::assemble;
pub(crate) use luma::LumaFrame;
pub use model::AnnotationPin;
#[cfg(test)]
pub(crate) use path::track_pin;
pub(crate) use path::{FrameSource, PinRequest};
pub(crate) use resolve::{displaced, fold, placement, PinnedPath};

/// The longest side frames are tracked at. Decoding costs the same at any
/// output size, since the hardware scales, and following costs about the
/// same, since it works on a fixed number of points; at 1920 a 5K Retina
/// recording is tracked at about one pixel a point, where the error against
/// a known scroll is under half a source pixel at the median.
pub(crate) const TRACKING_SIDE: u32 = 1920;

#[cfg(test)]
mod tests;
