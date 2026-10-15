// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scene clips applied to a frame: the preview and the export arrange each
//! frame through the one function here, so the two always agree. The clips
//! themselves, and the layouts they take, are `screenwide_scenes`.

pub mod arrange;
mod target;

pub use arrange::{arrange, camera_target};

#[cfg(test)]
mod custom_tests;
#[cfg(test)]
mod framing_tests;
#[cfg(test)]
mod motion_tests;
#[cfg(test)]
mod preset_tests;
#[cfg(test)]
mod radius_tests;
#[cfg(test)]
mod rotation_tests;
#[cfg(test)]
mod tests;
